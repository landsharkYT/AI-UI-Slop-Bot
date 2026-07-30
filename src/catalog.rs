#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleDefinition {
    pub id: &'static str,
    pub contract_version: &'static str,
    pub summary: &'static str,
    pub remediation: &'static str,
    pub requires_routes: bool,
    pub requires_house_style: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageArchetypeDefinition {
    pub id: &'static str,
    pub label: &'static str,
    pub keywords: &'static [&'static str],
}

const RULES: [RuleDefinition; 9] = [
    RuleDefinition {
        id: "repeated-decorative-shell",
        contract_version: "0.1.0-prototype",
        summary: "A rich decorative shell recurs across distinct component owners.",
        remediation: "Simplify or differentiate repeated container treatments while preserving hierarchy.",
        requires_routes: false,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "template-convergence",
        contract_version: "0.1.0-alpha",
        summary: "A route-owned page combines several stock structures into an interchangeable formula.",
        remediation: "Replace stock composition with product-specific hierarchy, interaction, or content shape.",
        requires_routes: true,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "effect-stacking",
        contract_version: "0.1.0-alpha",
        summary: "Several high-intensity decorative categories coexist on one reachable element.",
        remediation: "Remove or subordinate effects that do not carry hierarchy or interaction meaning.",
        requires_routes: false,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "decoration-saturation",
        contract_version: "0.1.0-alpha",
        summary: "One decorative treatment repeats until it overwhelms meaningful hierarchy.",
        remediation: "Reserve the treatment for hierarchy-bearing regions.",
        requires_routes: false,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "shape-homogenization",
        contract_version: "0.1.0-alpha",
        summary: "The same conspicuous silhouette is applied across structurally different roles.",
        remediation: "Restore role-specific silhouettes within the intended design language.",
        requires_routes: false,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "cardification",
        contract_version: "0.1.0-alpha",
        summary: "Nested or repetitive floating containers replace meaningful content grouping.",
        remediation: "Recover semantic grouping before simplifying card chrome.",
        requires_routes: false,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "generic-container-depth",
        contract_version: "0.1.0-alpha",
        summary: "A deep non-semantic wrapper chain participates in decorative layering.",
        remediation: "Flatten wrappers only where layout, focus, event, and behavior semantics remain intact.",
        requires_routes: false,
        requires_house_style: false,
    },
    RuleDefinition {
        id: "design-token-drift",
        contract_version: "0.1.0-alpha",
        summary: "Repeated visual values diverge from an explicit approved House Style scale.",
        remediation: "Use an approved token or deliberately review and add the value.",
        requires_routes: false,
        requires_house_style: true,
    },
    RuleDefinition {
        id: "rhythm-homogenization",
        contract_version: "0.1.0-alpha",
        summary: "Uniform spacing and sizing erase distinctions between different content roles.",
        remediation: "Introduce hierarchy-driven rhythm changes rather than arbitrary variation.",
        requires_routes: false,
        requires_house_style: false,
    },
];

const ARCHETYPES: [PageArchetypeDefinition; 14] = [
    PageArchetypeDefinition {
        id: "marketing",
        label: "Marketing and product landing",
        keywords: &["landing", "marketing", "home", "hero"],
    },
    PageArchetypeDefinition {
        id: "dashboard",
        label: "Dashboard and analytics",
        keywords: &["dashboard", "analytics", "metrics", "insights"],
    },
    PageArchetypeDefinition {
        id: "authentication",
        label: "Authentication and account recovery",
        keywords: &["login", "signin", "signup", "register", "password", "auth"],
    },
    PageArchetypeDefinition {
        id: "onboarding",
        label: "Onboarding and setup",
        keywords: &["onboarding", "setup", "welcome", "getting-started"],
    },
    PageArchetypeDefinition {
        id: "settings",
        label: "Settings, profile, and account management",
        keywords: &["settings", "profile", "account", "preferences"],
    },
    PageArchetypeDefinition {
        id: "pricing",
        label: "Pricing, plans, and billing",
        keywords: &["pricing", "plans", "billing", "subscription"],
    },
    PageArchetypeDefinition {
        id: "commerce",
        label: "Commerce catalog, product, cart, and checkout",
        keywords: &["shop", "catalog", "product", "cart", "checkout", "store"],
    },
    PageArchetypeDefinition {
        id: "portfolio",
        label: "Portfolio and showcase",
        keywords: &["portfolio", "showcase", "gallery", "work"],
    },
    PageArchetypeDefinition {
        id: "content",
        label: "Documentation, article, and content-heavy",
        keywords: &["docs", "documentation", "article", "blog", "guide"],
    },
    PageArchetypeDefinition {
        id: "administration",
        label: "Administrative and data management",
        keywords: &["admin", "management", "users", "inventory"],
    },
    PageArchetypeDefinition {
        id: "search",
        label: "Search and results",
        keywords: &["search", "results", "discover"],
    },
    PageArchetypeDefinition {
        id: "social",
        label: "Social, community, messaging, and activity",
        keywords: &[
            "social",
            "community",
            "messages",
            "chat",
            "activity",
            "feed",
        ],
    },
    PageArchetypeDefinition {
        id: "workflow",
        label: "Forms and multi-step workflows",
        keywords: &["form", "wizard", "workflow", "step", "apply"],
    },
    PageArchetypeDefinition {
        id: "status",
        label: "Empty, loading, error, and success states",
        keywords: &["empty", "loading", "error", "success", "not-found"],
    },
];

const STRUCTURAL_SIGNALS: [&str; 7] = [
    "eyebrow-pill",
    "centered-hero",
    "gradient-heading",
    "paired-cta",
    "framed-product-media",
    "bento-grid",
    "three-card-features",
];

#[must_use]
pub fn rule_catalog() -> &'static [RuleDefinition] {
    &RULES
}

#[must_use]
pub fn page_archetype_catalog() -> &'static [PageArchetypeDefinition] {
    &ARCHETYPES
}

#[must_use]
pub fn structural_signal_catalog() -> &'static [&'static str] {
    &STRUCTURAL_SIGNALS
}
