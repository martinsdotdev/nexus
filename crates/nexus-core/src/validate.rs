//! The validator: "explicit consistency" over the read model (the morphed
//! `decide` of ADR-0005). `validate` is pure and total over a [`Workspace`]; the
//! impure `apply_repairs` writes the corrective ops back into the `LoroDoc`.
//!
//! Invariant classes:
//! - **Repairable**: an active layout's `active_scene_id` points to no scene it
//!   owns -> reset to its first scene.
//! - **Strong**: an archived layout must hold no active scene -> blank it
//!   (archive wins over a concurrent activation).
//! - Weak invariants (concurrent activations of the same layout) need no repair:
//!   Loro's LWW already converges them.

use crate::model::Workspace;
use crate::schema;
use loro::LoroDoc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairClass {
    Repairable,
    Strong,
}

/// A corrective operation the relay applies after a merge to restore an
/// invariant. Plain data (no Loro handles) so the predicate stays portable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repair {
    pub layout_id: String,
    pub new_active_scene_id: String,
    pub class: RepairClass,
    pub reason: String,
}

/// Check the merged read model and return the corrective ops needed to restore
/// invariants. Pure and total: a malformed workspace yields repairs, never a panic.
pub fn validate(ws: &Workspace) -> Vec<Repair> {
    let mut repairs = Vec::new();
    for layout in &ws.layouts {
        if layout.status == "archived" {
            // Strong: archive wins over any concurrent activation.
            if !layout.active_scene_id.is_empty() {
                repairs.push(Repair {
                    layout_id: layout.id.clone(),
                    new_active_scene_id: String::new(),
                    class: RepairClass::Strong,
                    reason: "archived layout must hold no active scene".into(),
                });
            }
        } else {
            // Repairable: an active scene id must name a scene the layout owns.
            let owns_active = layout
                .scenes
                .iter()
                .any(|scene| scene.id == layout.active_scene_id);
            if !owns_active {
                let fallback = layout
                    .scenes
                    .first()
                    .map(|scene| scene.id.clone())
                    .unwrap_or_default();
                repairs.push(Repair {
                    layout_id: layout.id.clone(),
                    new_active_scene_id: fallback,
                    class: RepairClass::Repairable,
                    reason: "active scene missing; reset to first scene".into(),
                });
            }
        }
    }
    repairs
}

/// Write the repairs back into the document as `activeSceneId` ops. The relay
/// runs this after merging a peer update; the corrective ops then propagate to
/// every replica via the normal sync path. A no-op when there is nothing to fix.
pub fn apply_repairs(doc: &LoroDoc, repairs: &[Repair]) -> loro::LoroResult<()> {
    if repairs.is_empty() {
        return Ok(());
    }
    let tree = doc.get_tree(schema::TREE);
    for layout_id in tree.roots() {
        let key = layout_id.to_string();
        for repair in repairs.iter().filter(|repair| repair.layout_id == key) {
            tree.get_meta(layout_id)?
                .insert("activeSceneId", repair.new_active_scene_id.clone())?;
        }
    }
    doc.commit();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_doc::build_default;
    use crate::model::{Layout, Scene, read_workspace};

    fn scene(id: &str, kind: &str) -> Scene {
        Scene {
            id: id.into(),
            kind: kind.into(),
            name: kind.into(),
            theme_id: String::new(),
        }
    }

    fn layout(status: &str, active: &str) -> Layout {
        Layout {
            id: "L".into(),
            name: "Main".into(),
            status: status.into(),
            active_scene_id: active.into(),
            scenes: vec![scene("s0", "live"), scene("s1", "brb")],
        }
    }

    fn workspace(layout: Layout) -> Workspace {
        Workspace {
            active_layout_id: "L".into(),
            layouts: vec![layout],
        }
    }

    #[test]
    fn clean_workspace_needs_no_repair() {
        assert!(validate(&workspace(layout("active", "s0"))).is_empty());
    }

    #[test]
    fn dangling_active_scene_is_repairable_to_first() {
        let repairs = validate(&workspace(layout("active", "ghost")));
        assert_eq!(repairs.len(), 1);
        assert_eq!(repairs[0].class, RepairClass::Repairable);
        assert_eq!(repairs[0].new_active_scene_id, "s0");
    }

    #[test]
    fn archived_layout_must_not_hold_active_scene() {
        let repairs = validate(&workspace(layout("archived", "s0")));
        assert_eq!(repairs.len(), 1);
        assert_eq!(repairs[0].class, RepairClass::Strong);
        assert_eq!(repairs[0].new_active_scene_id, "");
    }

    #[test]
    fn apply_repairs_fixes_dangling_active_scene() {
        let doc = LoroDoc::new();
        build_default(&doc).unwrap();

        // Corrupt: point the active scene at an id the layout does not own.
        let tree = doc.get_tree(schema::TREE);
        let layout = tree.roots()[0];
        tree.get_meta(layout)
            .unwrap()
            .insert("activeSceneId", "ghost")
            .unwrap();
        doc.commit();

        let repairs = validate(&read_workspace(&doc));
        assert_eq!(repairs.len(), 1, "the dangling active scene is detected");
        apply_repairs(&doc, &repairs).unwrap();

        let ws = read_workspace(&doc);
        assert_eq!(
            ws.layouts[0].active_scene_id, ws.layouts[0].scenes[0].id,
            "repaired to the first scene"
        );
        assert!(validate(&ws).is_empty(), "no residual violations");
    }

    #[test]
    fn concurrent_activations_converge_by_lww() {
        // Shared default, snapshotted to a second replica.
        let doc1 = LoroDoc::new();
        doc1.set_peer_id(1).unwrap();
        build_default(&doc1).unwrap();
        let snapshot = doc1.export(loro::ExportMode::Snapshot).unwrap();

        let doc2 = LoroDoc::new();
        doc2.set_peer_id(2).unwrap();
        doc2.import(&snapshot).unwrap();

        let base = doc1.oplog_vv();

        let layout = doc1.get_tree(schema::TREE).roots()[0];
        let scenes = doc1.get_tree(schema::TREE).children(layout).unwrap();
        let scene_b = scenes[1].to_string();
        let scene_c = scenes[2].to_string();

        // Concurrent, conflicting activations of the same layout.
        doc1.get_tree(schema::TREE)
            .get_meta(layout)
            .unwrap()
            .insert("activeSceneId", scene_b.clone())
            .unwrap();
        doc1.commit();
        doc2.get_tree(schema::TREE)
            .get_meta(layout)
            .unwrap()
            .insert("activeSceneId", scene_c.clone())
            .unwrap();
        doc2.commit();

        // Exchange the divergent updates both ways.
        let up1 = doc1.export(loro::ExportMode::updates(&base)).unwrap();
        let up2 = doc2.export(loro::ExportMode::updates(&base)).unwrap();
        doc1.import(&up2).unwrap();
        doc2.import(&up1).unwrap();

        let a1 = read_workspace(&doc1).layouts[0].active_scene_id.clone();
        let a2 = read_workspace(&doc2).layouts[0].active_scene_id.clone();
        assert_eq!(a1, a2, "the two replicas converge");
        assert_eq!(a1, scene_c, "the higher peer id wins last-writer-wins");
        assert!(
            validate(&read_workspace(&doc1)).is_empty(),
            "the converged state is valid"
        );
    }
}
