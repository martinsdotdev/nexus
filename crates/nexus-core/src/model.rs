//! Plain read-model structs and a reader that projects the Loro document into
//! them. The validator works against these structs, not raw `LoroDoc` handles,
//! keeping the invariant predicates WASM-portable later (ADR-0005 / plan T7).

use crate::schema;
use loro::{LoroDoc, LoroMap, LoroValue, ValueOrContainer};

/// A read-only projection of the collaborative workspace document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub active_layout_id: String,
    pub layouts: Vec<Layout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub id: String,
    pub name: String,
    pub status: String,
    pub active_scene_id: String,
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub id: String,
    pub kind: String,
    pub name: String,
}

/// Read a string field from a node/workspace meta map, empty string if absent
/// or not a string. The reader is total: a malformed doc yields empty fields
/// that the validator then repairs.
fn map_str(map: &LoroMap, key: &str) -> String {
    match map.get(key) {
        Some(ValueOrContainer::Value(LoroValue::String(s))) => s.to_string(),
        _ => String::new(),
    }
}

/// Project the collaborative Loro document into the plain read model.
pub fn read_workspace(doc: &LoroDoc) -> Workspace {
    let workspace = doc.get_map(schema::WORKSPACE);
    let tree = doc.get_tree(schema::TREE);

    let layouts = tree
        .roots()
        .into_iter()
        .map(|layout_id| {
            let layout_meta = tree.get_meta(layout_id).expect("layout meta map");
            let scenes = tree
                .children(layout_id)
                .unwrap_or_default()
                .into_iter()
                .map(|scene_id| {
                    let scene_meta = tree.get_meta(scene_id).expect("scene meta map");
                    Scene {
                        id: scene_id.to_string(),
                        kind: map_str(&scene_meta, "kind"),
                        name: map_str(&scene_meta, "name"),
                    }
                })
                .collect();
            Layout {
                id: layout_id.to_string(),
                name: map_str(&layout_meta, "name"),
                status: map_str(&layout_meta, "status"),
                active_scene_id: map_str(&layout_meta, "activeSceneId"),
                scenes,
            }
        })
        .collect();

    Workspace {
        active_layout_id: map_str(&workspace, "activeLayoutId"),
        layouts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_doc::build_default;

    #[test]
    fn reads_curated_default() {
        let doc = LoroDoc::new();
        build_default(&doc).unwrap();
        let ws = read_workspace(&doc);

        assert_eq!(ws.layouts.len(), 1);
        let layout = &ws.layouts[0];
        assert_eq!(layout.name, "Main");
        assert_eq!(layout.status, "active");
        assert_eq!(ws.active_layout_id, layout.id);

        let kinds: Vec<&str> = layout.scenes.iter().map(|s| s.kind.as_str()).collect();
        assert_eq!(kinds, ["live", "starting_soon", "brb", "ending"]);

        // The default activates the live scene (the first one).
        assert_eq!(layout.active_scene_id, layout.scenes[0].id);
    }
}
