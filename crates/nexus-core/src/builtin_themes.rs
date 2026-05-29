//! The built-in themes, seeded into the workspace theme registry at first run
//! (ADR-0007, superseding the CSS `[data-theme]` files of ADR-0006). The relay
//! writes these as `protected` registry entries; every theme, built-in or
//! custom, is now plain data the editor applies as inline CSS variables. Values
//! are ported verbatim from the former `shared/styles/themes/*.css` files.

/// A built-in theme: a stable id (also the display name's lowercase form and the
/// fallback a dangling scene `themeId` resets to), a display name, and the
/// complete overlay-token vocabulary as `(token, value)` pairs.
pub struct BuiltinTheme {
    pub id: &'static str,
    pub name: &'static str,
    pub tokens: &'static [(&'static str, &'static str)],
}

/// The default theme: the always-present, protected registry entry every
/// dangling reference is repaired to.
pub const DEFAULT_THEME_ID: &str = "cozy";

/// The four built-in themes, in display order. Each carries the full 33-token
/// vocabulary so it stands alone (no inheritance layer needed, ADR-0007).
pub const BUILTIN_THEMES: &[BuiltinTheme] = &[
    BuiltinTheme {
        id: "cozy",
        name: "Cozy",
        tokens: &[
            ("background", "oklch(95% 0.02 80)"),
            ("foreground", "oklch(15% 0.02 80)"),
            ("card", "oklch(98% 0.02 80)"),
            ("card-foreground", "oklch(15% 0.02 80)"),
            ("popover", "oklch(98% 0.02 80)"),
            ("popover-foreground", "oklch(15% 0.02 80)"),
            ("muted", "oklch(92% 0.02 80)"),
            ("muted-foreground", "oklch(45% 0.02 80)"),
            ("primary", "oklch(70% 0.15 50)"),
            ("primary-foreground", "oklch(15% 0 0)"),
            ("secondary", "oklch(90% 0.02 80)"),
            ("secondary-foreground", "oklch(15% 0.02 80)"),
            ("accent", "oklch(88% 0.04 50)"),
            ("accent-foreground", "oklch(15% 0.02 80)"),
            ("info", "oklch(65% 0.14 230)"),
            ("info-foreground", "oklch(98% 0 0)"),
            ("success", "oklch(65% 0.16 145)"),
            ("success-foreground", "oklch(98% 0 0)"),
            ("warning", "oklch(78% 0.16 90)"),
            ("warning-foreground", "oklch(15% 0 0)"),
            ("destructive", "oklch(58% 0.2 30)"),
            ("destructive-foreground", "oklch(98% 0 0)"),
            ("border", "oklch(85% 0.02 80)"),
            ("border-subtle", "oklch(90% 0.02 80)"),
            ("input", "oklch(94% 0.02 80)"),
            ("ring", "oklch(70% 0.15 50)"),
            ("invert", "oklch(20% 0.02 80)"),
            ("invert-foreground", "oklch(95% 0.02 80)"),
            ("radius", "10px"),
            ("shadow-widget", "0 4px 12px oklch(40% 0.05 80 / 0.1)"),
            ("font-display", "'Bricolage Grotesque', sans-serif"),
            ("font-body", "'Geist', sans-serif"),
            ("motion-feel", "'gentle'"),
        ],
    },
    BuiltinTheme {
        id: "cyber",
        name: "Cyber",
        tokens: &[
            ("background", "oklch(18% 0.03 250)"),
            ("foreground", "oklch(95% 0.02 200)"),
            ("card", "oklch(24% 0.04 250)"),
            ("card-foreground", "oklch(95% 0.02 200)"),
            ("popover", "oklch(22% 0.04 250)"),
            ("popover-foreground", "oklch(95% 0.02 200)"),
            ("muted", "oklch(30% 0.03 250)"),
            ("muted-foreground", "oklch(72% 0.05 200)"),
            ("primary", "oklch(75% 0.2 200)"),
            ("primary-foreground", "oklch(15% 0.03 250)"),
            ("secondary", "oklch(30% 0.04 290)"),
            ("secondary-foreground", "oklch(95% 0.02 200)"),
            ("accent", "oklch(70% 0.22 320)"),
            ("accent-foreground", "oklch(15% 0.03 250)"),
            ("info", "oklch(72% 0.16 230)"),
            ("info-foreground", "oklch(15% 0 0)"),
            ("success", "oklch(78% 0.2 150)"),
            ("success-foreground", "oklch(15% 0 0)"),
            ("warning", "oklch(82% 0.18 95)"),
            ("warning-foreground", "oklch(15% 0 0)"),
            ("destructive", "oklch(65% 0.24 15)"),
            ("destructive-foreground", "oklch(98% 0 0)"),
            ("border", "oklch(45% 0.08 200)"),
            ("border-subtle", "oklch(34% 0.05 250)"),
            ("input", "oklch(28% 0.04 250)"),
            ("ring", "oklch(75% 0.2 200)"),
            ("invert", "oklch(95% 0.02 200)"),
            ("invert-foreground", "oklch(18% 0.03 250)"),
            ("radius", "2px"),
            ("shadow-widget", "0 0 16px oklch(75% 0.2 200 / 0.35)"),
            ("font-display", "'Orbitron', 'Share Tech Mono', monospace"),
            ("font-body", "'Share Tech Mono', monospace"),
            ("motion-feel", "'snappy'"),
        ],
    },
    BuiltinTheme {
        id: "editorial",
        name: "Editorial",
        tokens: &[
            ("background", "oklch(99% 0 0)"),
            ("foreground", "oklch(12% 0 0)"),
            ("card", "oklch(99% 0 0)"),
            ("card-foreground", "oklch(12% 0 0)"),
            ("popover", "oklch(99% 0 0)"),
            ("popover-foreground", "oklch(12% 0 0)"),
            ("muted", "oklch(94% 0 0)"),
            ("muted-foreground", "oklch(40% 0 0)"),
            ("primary", "oklch(12% 0 0)"),
            ("primary-foreground", "oklch(99% 0 0)"),
            ("secondary", "oklch(94% 0 0)"),
            ("secondary-foreground", "oklch(12% 0 0)"),
            ("accent", "oklch(60% 0.22 25)"),
            ("accent-foreground", "oklch(99% 0 0)"),
            ("info", "oklch(45% 0.1 250)"),
            ("info-foreground", "oklch(99% 0 0)"),
            ("success", "oklch(45% 0.12 150)"),
            ("success-foreground", "oklch(99% 0 0)"),
            ("warning", "oklch(70% 0.15 85)"),
            ("warning-foreground", "oklch(12% 0 0)"),
            ("destructive", "oklch(50% 0.22 25)"),
            ("destructive-foreground", "oklch(99% 0 0)"),
            ("border", "oklch(12% 0 0)"),
            ("border-subtle", "oklch(85% 0 0)"),
            ("input", "oklch(96% 0 0)"),
            ("ring", "oklch(12% 0 0)"),
            ("invert", "oklch(12% 0 0)"),
            ("invert-foreground", "oklch(99% 0 0)"),
            ("radius", "0px"),
            ("shadow-widget", "none"),
            ("font-display", "'Playfair Display', Georgia, serif"),
            ("font-body", "Georgia, serif"),
            ("motion-feel", "'crisp'"),
        ],
    },
    BuiltinTheme {
        id: "sticker",
        name: "Sticker",
        tokens: &[
            ("background", "oklch(97% 0.05 320)"),
            ("foreground", "oklch(20% 0.08 300)"),
            ("card", "oklch(99% 0.03 320)"),
            ("card-foreground", "oklch(20% 0.08 300)"),
            ("popover", "oklch(99% 0.03 320)"),
            ("popover-foreground", "oklch(20% 0.08 300)"),
            ("muted", "oklch(92% 0.06 320)"),
            ("muted-foreground", "oklch(50% 0.1 320)"),
            ("primary", "oklch(70% 0.24 350)"),
            ("primary-foreground", "oklch(99% 0 0)"),
            ("secondary", "oklch(85% 0.15 200)"),
            ("secondary-foreground", "oklch(20% 0.08 300)"),
            ("accent", "oklch(82% 0.2 130)"),
            ("accent-foreground", "oklch(20% 0.08 300)"),
            ("info", "oklch(70% 0.18 230)"),
            ("info-foreground", "oklch(99% 0 0)"),
            ("success", "oklch(75% 0.2 150)"),
            ("success-foreground", "oklch(20% 0 0)"),
            ("warning", "oklch(85% 0.18 90)"),
            ("warning-foreground", "oklch(20% 0 0)"),
            ("destructive", "oklch(65% 0.24 20)"),
            ("destructive-foreground", "oklch(99% 0 0)"),
            ("border", "oklch(20% 0.08 300)"),
            ("border-subtle", "oklch(88% 0.06 320)"),
            ("input", "oklch(96% 0.04 320)"),
            ("ring", "oklch(70% 0.24 350)"),
            ("invert", "oklch(20% 0.08 300)"),
            ("invert-foreground", "oklch(97% 0.05 320)"),
            ("radius", "18px"),
            ("shadow-widget", "0 6px 0 oklch(20% 0.08 300 / 0.2)"),
            ("font-display", "'Fredoka', 'Baloo 2', sans-serif"),
            ("font-body", "'Nunito', sans-serif"),
            ("motion-feel", "'bouncy'"),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The canonical 33-token vocabulary, a mirror of THEME_TOKENS in
    /// `entities/theme/model/tokens.ts`. A built-in that typos a token name (still
    /// 33 entries, so a count check passes) leaves a canonical token unset and
    /// emits an unknown one; this guards against that.
    const VOCABULARY: &[&str] = &[
        "background",
        "foreground",
        "card",
        "card-foreground",
        "popover",
        "popover-foreground",
        "muted",
        "muted-foreground",
        "primary",
        "primary-foreground",
        "secondary",
        "secondary-foreground",
        "accent",
        "accent-foreground",
        "info",
        "info-foreground",
        "success",
        "success-foreground",
        "warning",
        "warning-foreground",
        "destructive",
        "destructive-foreground",
        "border",
        "border-subtle",
        "input",
        "ring",
        "invert",
        "invert-foreground",
        "radius",
        "shadow-widget",
        "font-display",
        "font-body",
        "motion-feel",
    ];

    #[test]
    fn every_builtin_defines_exactly_the_canonical_vocabulary() {
        let vocabulary: BTreeSet<&str> = VOCABULARY.iter().copied().collect();
        assert_eq!(vocabulary.len(), 33, "the vocabulary has 33 unique tokens");
        for theme in BUILTIN_THEMES {
            let names: BTreeSet<&str> = theme.tokens.iter().map(|&(key, _)| key).collect();
            assert_eq!(
                names.len(),
                theme.tokens.len(),
                "{} has no duplicate token names",
                theme.id
            );
            assert_eq!(
                names, vocabulary,
                "{} defines exactly the vocabulary",
                theme.id
            );
        }
    }
}
