import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class QualificationSpecsTest(unittest.TestCase):
    def test_frozen_protocol_matches_v1_gate_counts(self) -> None:
        protocol = json.loads(
            (ROOT / "qualification" / "protocol.json").read_text(encoding="utf-8")
        )

        self.assertEqual(protocol["status"], "frozen-for-v1-qualification")
        self.assertEqual(len(protocol["rules"]["ids"]), 11)
        self.assertEqual(protocol["rules"]["holdoutPerRule"]["minimumPositiveCases"], 20)
        self.assertEqual(
            protocol["rules"]["holdoutPerRule"]["minimumAcceptableCounterexamples"], 20
        )
        self.assertEqual(len(protocol["archetypes"]["ids"]), 14)
        self.assertEqual(protocol["maintainerTrial"]["externalReactMaintainers"], 7)
        self.assertEqual(protocol["maintainerTrial"]["minimumAllThreePositiveRatings"], 5)
        self.assertEqual(protocol["agentTrial"]["minimumFreshTrials"], 10)
        self.assertEqual(protocol["agentTrial"]["minimumPassingTrials"], 8)
        self.assertEqual(
            protocol["automatedQualification"]["minimumMutationScorePercent"], 80
        )
        self.assertEqual(
            len(protocol["automatedQualification"]["nativeTargets"]), 5
        )

    def test_reference_runner_records_reproducibility_fields(self) -> None:
        runner = json.loads(
            (ROOT / "qualification" / "reference-runner.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(runner["status"], "frozen-for-v1-qualification")
        self.assertEqual(runner["runnerLabel"], "ubuntu-24.04")
        self.assertEqual(runner["cpuAllocation"]["logicalProcessors"], 4)
        self.assertIn("resolvedImageVersion", runner["requiredEvidence"])
        self.assertIn("rulePackVersion", runner["requiredEvidence"])
        self.assertIn("peakRssKiB", runner["requiredEvidence"])


if __name__ == "__main__":
    unittest.main()
