use std::{
    collections::{BTreeMap, BTreeSet},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{CanonicalReport, Finding};

pub const BASELINE_SCHEMA_VERSION: &str = "2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineArtifact {
    pub artifact_type: String,
    pub schema_version: String,
    pub status: BaselineStatus,
    pub tool_version: String,
    pub rule_pack_version: String,
    pub fingerprint_algorithm_version: String,
    pub evidence_digest_algorithm_version: String,
    pub policy_fingerprints: BTreeMap<String, String>,
    pub findings: Vec<BaselineFinding>,
    pub review: Option<BaselineReview>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BaselineStatus {
    Candidate,
    Reviewed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineFinding {
    pub scope_id: String,
    pub fingerprint: String,
    pub evidence_digest: String,
    pub rule_id: String,
    pub path: String,
    pub owner: String,
    pub score: u8,
    pub band: String,
    pub confidence: String,
    pub signature: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineReview {
    pub approver: String,
    pub accepted_at: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineComparison {
    pub status: String,
    pub compatible: bool,
    pub changes: Vec<BaselineChange>,
    pub enforceable_regression_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineChange {
    pub kind: String,
    pub scope_id: String,
    pub fingerprint: String,
    pub rule_id: String,
    pub path: String,
    pub owner: String,
    pub previous_score: Option<u8>,
    pub current_score: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineMigrationPreview {
    pub artifact_type: String,
    pub schema_version: String,
    pub compatible: bool,
    pub changes: Vec<BaselineChange>,
    pub ambiguous_count: usize,
}

#[must_use]
pub fn create_candidate(report: &CanonicalReport) -> BaselineArtifact {
    let mut findings = Vec::new();
    let mut policy_fingerprints = BTreeMap::new();
    for scope in &report.scopes {
        policy_fingerprints.insert(scope.id.clone(), scope.policy_fingerprint.clone());
        for finding in &scope.findings {
            if finding.policy_disposition != "suppress" {
                findings.push(to_baseline_finding(&scope.id, finding));
            }
        }
    }
    findings.sort_by(|left, right| {
        (
            &left.scope_id,
            &left.path,
            &left.owner,
            &left.rule_id,
            &left.fingerprint,
        )
            .cmp(&(
                &right.scope_id,
                &right.path,
                &right.owner,
                &right.rule_id,
                &right.fingerprint,
            ))
    });
    BaselineArtifact {
        artifact_type: "ai-ui-slop.baseline".to_owned(),
        schema_version: BASELINE_SCHEMA_VERSION.to_owned(),
        status: BaselineStatus::Candidate,
        tool_version: report.tool_version.clone(),
        rule_pack_version: report.rule_pack_version.clone(),
        fingerprint_algorithm_version: report.fingerprint_algorithm_version.clone(),
        evidence_digest_algorithm_version: report.evidence_digest_algorithm_version.clone(),
        policy_fingerprints,
        findings,
        review: None,
    }
}

pub fn accept_candidate(
    mut candidate: BaselineArtifact,
    approver: &str,
    rationale: &str,
) -> Result<BaselineArtifact, String> {
    if candidate.status != BaselineStatus::Candidate {
        return Err("only an unreviewed baseline candidate can be accepted".to_owned());
    }
    if approver.trim().is_empty() || rationale.trim().is_empty() {
        return Err("baseline acceptance requires an approver and rationale".to_owned());
    }
    let accepted_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    candidate.status = BaselineStatus::Reviewed;
    candidate.review = Some(BaselineReview {
        approver: approver.trim().to_owned(),
        accepted_at: format!("unix:{accepted_at}"),
        rationale: rationale.trim().to_owned(),
    });
    Ok(candidate)
}

#[must_use]
pub fn compare_baseline(
    report: &CanonicalReport,
    baseline: &BaselineArtifact,
) -> BaselineComparison {
    let compatible = baseline.status == BaselineStatus::Reviewed
        && baseline.schema_version == BASELINE_SCHEMA_VERSION
        && baseline.rule_pack_version == report.rule_pack_version
        && baseline.fingerprint_algorithm_version == report.fingerprint_algorithm_version
        && baseline.evidence_digest_algorithm_version == report.evidence_digest_algorithm_version
        && report.scopes.iter().all(|scope| {
            baseline
                .policy_fingerprints
                .get(&scope.id)
                .is_some_and(|fingerprint| fingerprint == &scope.policy_fingerprint)
        });
    if !compatible {
        return BaselineComparison {
            status: "incompatible".to_owned(),
            compatible: false,
            changes: Vec::new(),
            enforceable_regression_count: 0,
        };
    }

    let previous = baseline
        .findings
        .iter()
        .map(|finding| {
            (
                (finding.scope_id.clone(), finding.fingerprint.clone()),
                finding,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut current = BTreeMap::<(String, String), (&Finding, bool)>::new();
    for scope in &report.scopes {
        for finding in &scope.findings {
            current.insert(
                (scope.id.clone(), finding.fingerprint.clone()),
                (finding, finding.policy_disposition == "enforce"),
            );
        }
    }
    let keys = previous
        .keys()
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut changes = Vec::new();
    let mut enforceable_regression_count = 0;
    for key in keys {
        let old = previous.get(&key).copied();
        let new = current.get(&key).copied();
        let (kind, enforceable) = match (old, new) {
            (None, Some((_, enforce))) => ("new", enforce),
            (Some(_), None) => ("resolved", false),
            (Some(old), Some((new, enforce))) if materially_worsened(old, new) => {
                ("worsened", enforce)
            }
            (Some(old), Some((new, _))) if new.score < old.score => ("improved", false),
            (Some(old), Some((new, _))) if new.evidence_digest != old.evidence_digest => {
                ("changed", false)
            }
            _ => continue,
        };
        if enforceable {
            enforceable_regression_count += 1;
        }
        let (rule_id, path, owner) = match (old, new) {
            (_, Some((finding, _))) => (
                finding.rule_id.clone(),
                finding.path.clone(),
                finding.owner.clone(),
            ),
            (Some(finding), None) => (
                finding.rule_id.clone(),
                finding.path.clone(),
                finding.owner.clone(),
            ),
            (None, None) => unreachable!(),
        };
        changes.push(BaselineChange {
            kind: kind.to_owned(),
            scope_id: key.0,
            fingerprint: key.1,
            rule_id,
            path,
            owner,
            previous_score: old.map(|finding| finding.score),
            current_score: new.map(|(finding, _)| finding.score),
        });
    }
    BaselineComparison {
        status: if enforceable_regression_count > 0 {
            "regression"
        } else if changes.is_empty() {
            "unchanged"
        } else {
            "changed"
        }
        .to_owned(),
        compatible: true,
        changes,
        enforceable_regression_count,
    }
}

#[must_use]
pub fn preview_baseline_migration(
    report: &CanonicalReport,
    baseline: &BaselineArtifact,
) -> BaselineMigrationPreview {
    let comparison = compare_baseline(report, baseline);
    if comparison.compatible {
        return BaselineMigrationPreview {
            artifact_type: "ai-ui-slop.baseline-preview".to_owned(),
            schema_version: BASELINE_SCHEMA_VERSION.to_owned(),
            compatible: true,
            changes: comparison.changes,
            ambiguous_count: 0,
        };
    }

    type SemanticKey = (String, String, String, String);
    let mut previous = BTreeMap::<SemanticKey, Vec<&BaselineFinding>>::new();
    for finding in &baseline.findings {
        previous
            .entry((
                finding.scope_id.clone(),
                finding.rule_id.clone(),
                finding.path.clone(),
                finding.owner.clone(),
            ))
            .or_default()
            .push(finding);
    }
    let mut current = BTreeMap::<SemanticKey, Vec<&Finding>>::new();
    for scope in &report.scopes {
        for finding in &scope.findings {
            current
                .entry((
                    scope.id.clone(),
                    finding.rule_id.clone(),
                    finding.path.clone(),
                    finding.owner.clone(),
                ))
                .or_default()
                .push(finding);
        }
    }
    let keys = previous
        .keys()
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut changes = Vec::new();
    let mut ambiguous_count = 0;
    for key in keys {
        let old = previous.get(&key).map(Vec::as_slice).unwrap_or_default();
        let new = current.get(&key).map(Vec::as_slice).unwrap_or_default();
        let kind = if old.len() > 1 || new.len() > 1 {
            ambiguous_count += 1;
            "ambiguous"
        } else {
            match (old.first(), new.first()) {
                (None, Some(_)) => "added",
                (Some(_), None) => "removed",
                (Some(old), Some(new))
                    if old.fingerprint == new.fingerprint
                        && old.evidence_digest == new.evidence_digest
                        && old.score == new.score =>
                {
                    continue;
                }
                (Some(_), Some(_)) => "changed",
                (None, None) => continue,
            }
        };
        let old = old.first().copied();
        let new = new.first().copied();
        changes.push(BaselineChange {
            kind: kind.to_owned(),
            scope_id: key.0,
            fingerprint: new
                .map(|finding| finding.fingerprint.clone())
                .or_else(|| old.map(|finding| finding.fingerprint.clone()))
                .unwrap_or_default(),
            rule_id: key.1,
            path: key.2,
            owner: key.3,
            previous_score: old.map(|finding| finding.score),
            current_score: new.map(|finding| finding.score),
        });
    }
    BaselineMigrationPreview {
        artifact_type: "ai-ui-slop.baseline-preview".to_owned(),
        schema_version: BASELINE_SCHEMA_VERSION.to_owned(),
        compatible: false,
        changes,
        ambiguous_count,
    }
}

fn materially_worsened(previous: &BaselineFinding, current: &Finding) -> bool {
    band_rank(&current.band) > band_rank(&previous.band)
        || current.score >= previous.score.saturating_add(10)
}

fn band_rank(band: &str) -> u8 {
    match band {
        "minimal" => 0,
        "low" => 1,
        "moderate" => 2,
        "high" => 3,
        "dominant" => 4,
        _ => 5,
    }
}

fn to_baseline_finding(scope_id: &str, finding: &Finding) -> BaselineFinding {
    BaselineFinding {
        scope_id: scope_id.to_owned(),
        fingerprint: finding.fingerprint.clone(),
        evidence_digest: finding.evidence_digest.clone(),
        rule_id: finding.rule_id.clone(),
        path: finding.path.clone(),
        owner: finding.owner.clone(),
        score: finding.score,
        band: finding.band.clone(),
        confidence: finding.confidence.clone(),
        signature: finding.signature.clone(),
        rationale: finding.explanation.clone(),
    }
}
