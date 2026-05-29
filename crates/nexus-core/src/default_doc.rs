//! The curated default workspace.
//!
//! Built once, on the relay, at first run and distributed to clients as a Loro
//! snapshot. Clients never construct it independently: independent construction
//! would mint divergent `TreeID`s that duplicate on merge (see ADR-0005 / the
//! plan, trap T1).

use crate::schema;
use loro::LoroDoc;

/// Build the curated default workspace into a fresh [`LoroDoc`].
pub fn build_default(doc: &LoroDoc) -> loro::LoroResult<()> {
    let tree = doc.get_tree(schema::TREE);

    let layout = tree.create(loro::TreeParentId::Root)?;
    let layout_meta = tree.get_meta(layout)?;
    layout_meta.insert("type", "layout")?;
    layout_meta.insert("name", "Main")?;
    layout_meta.insert("status", "active")?;

    let mut first_scene = None;
    for kind in ["live", "starting_soon", "brb", "ending"] {
        let scene = tree.create(layout)?;
        let scene_meta = tree.get_meta(scene)?;
        scene_meta.insert("type", "scene")?;
        scene_meta.insert("kind", kind)?;
        scene_meta.insert("name", kind)?;
        if first_scene.is_none() {
            first_scene = Some(scene);
        }
    }
    layout_meta.insert(
        "activeSceneId",
        first_scene.expect("four scenes were created").to_string(),
    )?;

    let workspace = doc.get_map(schema::WORKSPACE);
    workspace.insert("activeLayoutId", layout.to_string())?;
    workspace.insert("schemaVersion", 1)?;

    doc.commit();
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
}
