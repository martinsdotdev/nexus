//! The curated default workspace.
//!
//! Built once, on the relay, at first run and distributed to clients as a Loro
//! snapshot. Clients never construct it independently: independent construction
//! would mint divergent `TreeID`s that duplicate on merge (see ADR-0005 / the
//! plan, trap T1).

use crate::builtin_themes::BUILTIN_THEMES;
use crate::schema;
use loro::{LoroDoc, LoroMap, LoroTree, TreeID};

/// Build the curated default workspace into a fresh [`LoroDoc`].
pub fn build_default(doc: &LoroDoc) -> loro::LoroResult<()> {
    let tree = doc.get_tree(schema::TREE);

    let layout = tree.create(loro::TreeParentId::Root)?;
    let layout_meta = tree.get_meta(layout)?;
    layout_meta.insert("type", "layout")?;
    layout_meta.insert("name", "Main")?;
    layout_meta.insert("status", "active")?;

    // Each scene gets a distinct theme so switching scenes in the editor
    // re-themes the overlay live (and exercises all four themes).
    let mut live_scene = None;
    for (kind, theme) in [
        ("live", "cozy"),
        ("starting_soon", "cyber"),
        ("brb", "editorial"),
        ("ending", "sticker"),
    ] {
        let scene = tree.create(layout)?;
        let scene_meta = tree.get_meta(scene)?;
        scene_meta.insert("type", "scene")?;
        scene_meta.insert("kind", kind)?;
        scene_meta.insert("name", kind)?;
        scene_meta.insert("themeId", theme)?;
        if live_scene.is_none() {
            live_scene = Some(scene);
        }
    }
    let live_scene = live_scene.expect("four scenes were created");
    layout_meta.insert("activeSceneId", live_scene.to_string())?;

    seed_default_widgets(&tree, live_scene)?;
    ensure_builtin_themes(doc)?;

    let workspace = doc.get_map(schema::WORKSPACE);
    workspace.insert("activeLayoutId", layout.to_string())?;
    workspace.insert("schemaVersion", 1)?;

    doc.commit();
    Ok(())
}

/// Ensure every built-in theme exists in the root "themes" registry (ADR-0007),
/// seeding any that are missing. Each is a nested map `{ name, base, protected,
/// tokens }`; built-ins carry no base (they stand alone) and are `protected` so a
/// user cannot delete the default fallback. Idempotent: an entry already present
/// is left untouched. Returns whether anything was added, so the caller commits +
/// persists only on change. This both seeds a fresh document and migrates one that
/// predates the seeded built-ins. Done only on the relay so the stable ids never
/// collide on merge (T1); the caller commits.
pub fn ensure_builtin_themes(doc: &LoroDoc) -> loro::LoroResult<bool> {
    let registry = doc.get_map(schema::THEMES);
    let mut added = false;
    for theme in BUILTIN_THEMES {
        if registry.get(theme.id).is_some() {
            continue;
        }
        let entry = registry.insert_container(theme.id, LoroMap::new())?;
        entry.insert("name", theme.name)?;
        entry.insert("base", "")?;
        entry.insert("protected", true)?;
        let tokens = entry.insert_container("tokens", LoroMap::new())?;
        for &(key, value) in theme.tokens {
            tokens.insert(key, value)?;
        }
        added = true;
    }
    Ok(added)
}

/// Seed the eight v1 widgets into the live scene at sensible positions on the
/// 1920x1080 virtual canvas. Each widget is a child node of the scene whose meta
/// carries its type, geometry, visibility, and default props.
fn seed_default_widgets(tree: &LoroTree, scene: TreeID) -> loro::LoroResult<()> {
    // (widgetType, x, y, w, h, z)
    let widgets = [
        ("webcam-frame", 40, 40, 480, 360, 1),
        ("alerts", 560, 60, 800, 200, 5),
        ("chat-box", 1460, 120, 420, 640, 1),
        ("follower-bubble", 40, 430, 440, 84, 2),
        ("stream-info", 40, 900, 560, 96, 2),
        ("now-playing", 1460, 940, 420, 100, 2),
        ("goal-bar", 620, 940, 520, 56, 2),
        ("socials", 620, 1010, 520, 48, 2),
    ];
    for (widget_type, x, y, w, h, z) in widgets {
        let node = tree.create(scene)?;
        let meta = tree.get_meta(node)?;
        meta.insert("type", "widget")?;
        meta.insert("widgetType", widget_type)?;
        meta.insert("x", x)?;
        meta.insert("y", y)?;
        meta.insert("w", w)?;
        meta.insert("h", h)?;
        meta.insert("z", z)?;
        meta.insert("visible", true)?;
        seed_widget_props(&meta, widget_type)?;
    }
    Ok(())
}

/// Default props per widget type. Prop-driven widgets get starter content;
/// event-driven widgets (chat-box, alerts, follower-bubble) render from the
/// event bus and need none.
fn seed_widget_props(meta: &LoroMap, widget_type: &str) -> loro::LoroResult<()> {
    match widget_type {
        "stream-info" => {
            meta.insert("title", "My Stream")?;
            meta.insert("game", "Just Chatting")?;
        }
        "goal-bar" => {
            meta.insert("label", "Follower Goal")?;
            meta.insert("current", 0)?;
            meta.insert("target", 100)?;
        }
        "socials" => {
            meta.insert("handles", "@nexus")?;
        }
        "now-playing" => {
            meta.insert("track", "Untitled")?;
            meta.insert("artist", "Unknown Artist")?;
        }
        "webcam-frame" => {
            meta.insert("shape", "squircle")?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_doc_has_one_layout_with_four_scenes() {
        let doc = LoroDoc::new();
        build_default(&doc).unwrap();

        let tree = doc.get_tree(schema::TREE);
        let layouts = tree.roots();
        assert_eq!(layouts.len(), 1, "one Main layout at the tree root");

        let scenes = tree.children(layouts[0]).expect("the layout has children");
        assert_eq!(scenes.len(), 4, "four scenes under the layout");
    }

    #[test]
    fn seeds_the_builtin_theme_registry() {
        let doc = LoroDoc::new();
        build_default(&doc).unwrap();

        let registry = doc.get_map(schema::THEMES);
        assert_eq!(
            registry.len(),
            4,
            "four built-in themes seeded into the registry"
        );
    }

    #[test]
    fn ensure_builtin_themes_is_idempotent() {
        let doc = LoroDoc::new();
        // A registry missing the built-ins (a pre-ADR-0007 document): all are added.
        assert!(ensure_builtin_themes(&doc).unwrap(), "seeds when missing");
        doc.commit();
        assert_eq!(doc.get_map(schema::THEMES).len(), 4);
        // A second pass finds them all present and changes nothing.
        assert!(!ensure_builtin_themes(&doc).unwrap(), "no-op when present");
        assert_eq!(doc.get_map(schema::THEMES).len(), 4);
    }
}
