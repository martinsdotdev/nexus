//! The validator: "explicit consistency" over the read model (the morphed
//! `decide` of ADR-0005). `validate` is pure and total over a [`Workspace`]; the
//! impure `apply_repairs` writes the corrective ops back into the `LoroDoc`.
//!
//! Invariant classes:
//! - **Repairable**: an active layout's `active_scene_id` points to no scene it
//!   owns -> reset to its first scene; a scene's `theme_id` names no theme in the
//!   registry (built-in or custom, ADR-0007) -> reset to the default theme.
//! - **Strong**: an archived layout must hold no active scene -> blank it
//!   (archive wins over a concurrent activation).
//! - Weak invariants (concurrent activations of the same layout) need no repair:
//!   Loro's LWW already converges them.

use std::collections::HashSet;

use crate::builtin_themes::DEFAULT_THEME_ID;
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
pub enum Repair {
    /// Set a layout's `activeSceneId` (reset a dangling one, or blank an
    /// archived layout's).
    SetActiveScene {
        layout_id: String,
        scene_id: String,
        class: RepairClass,
        reason: String,
    },
    /// Reset a scene's `themeId` that references a theme that no longer exists.
    SetSceneTheme {
        scene_id: String,
        theme_id: String,
        reason: String,
    },
    /// Reset the workspace's `activeLayoutId` when it names a layout that no
    /// longer exists (e.g. the active layout was deleted).
    SetActiveLayout { layout_id: String, reason: String },
}

/// Check the merged read model and return the corrective ops needed to restore
/// invariants. Pure and total: a malformed workspace yields repairs, never a panic.
pub fn validate(ws: &Workspace) -> Vec<Repair> {
    let mut repairs = Vec::new();

    for layout in &ws.layouts {
        if layout.status == "archived" {
            // Strong: archive wins over any concurrent activation.
            if !layout.active_scene_id.is_empty() {
                repairs.push(Repair::SetActiveScene {
                    layout_id: layout.id.clone(),
                    scene_id: String::new(),
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
                repairs.push(Repair::SetActiveScene {
                    layout_id: layout.id.clone(),
                    scene_id: fallback,
                    class: RepairClass::Repairable,
                    reason: "active scene missing; reset to first scene".into(),
                });
            }
        }
    }

    // Repairable: a scene's theme must name a theme in the registry (the seeded
    // built-ins and any custom themes; ADR-0007).
    let known: HashSet<&str> = ws.themes.iter().map(|theme| theme.id.as_str()).collect();
    for layout in &ws.layouts {
        for scene in &layout.scenes {
            if !scene.theme_id.is_empty() && !known.contains(scene.theme_id.as_str()) {
                repairs.push(Repair::SetSceneTheme {
                    scene_id: scene.id.clone(),
                    theme_id: DEFAULT_THEME_ID.into(),
                    reason: "theme missing; reset to default".into(),
                });
            }
        }
    }

    // Repairable: the active layout id must name a layout that still exists; if it
    // names none (e.g. the active layout was deleted), reset to the first active
    // layout, or the first layout if all happen to be archived.
    if !ws
        .layouts
        .iter()
        .any(|layout| layout.id == ws.active_layout_id)
    {
        let fallback = ws
            .layouts
            .iter()
            .find(|layout| layout.status != "archived")
            .or_else(|| ws.layouts.first())
            .map(|layout| layout.id.clone())
            .unwrap_or_default();
        repairs.push(Repair::SetActiveLayout {
            layout_id: fallback,
            reason: "active layout missing; reset to first".into(),
        });
    }

    repairs
}

/// Write the repairs back into the document. The relay runs this after merging a
/// peer update; the corrective ops then propagate to every replica via the normal
/// sync path. A no-op when there is nothing to fix.
pub fn apply_repairs(doc: &LoroDoc, repairs: &[Repair]) -> loro::LoroResult<()> {
    if repairs.is_empty() {
        return Ok(());
    }
    let tree = doc.get_tree(schema::TREE);
    for repair in repairs {
        match repair {
            Repair::SetActiveScene {
                layout_id,
                scene_id,
                ..
            } => {
                for root in tree.roots() {
                    if root.to_string() == *layout_id {
                        tree.get_meta(root)?
                            .insert("activeSceneId", scene_id.clone())?;
                    }
                }
            }
            Repair::SetSceneTheme {
                scene_id, theme_id, ..
            } => {
                for root in tree.roots() {
                    for child in tree.children(root).unwrap_or_default() {
                        if child.to_string() == *scene_id {
                            tree.get_meta(child)?.insert("themeId", theme_id.clone())?;
                        }
                    }
                }
            }
            Repair::SetActiveLayout { layout_id, .. } => {
                doc.get_map(schema::WORKSPACE)
                    .insert("activeLayoutId", layout_id.clone())?;
            }
        }
    }
    doc.commit();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_doc::build_default;
    use crate::model::{Layout, Scene, ThemeDef, read_workspace};

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
            themes: Vec::new(),
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
        match &repairs[0] {
            Repair::SetActiveScene {
                class, scene_id, ..
            } => {
                assert_eq!(*class, RepairClass::Repairable);
                assert_eq!(scene_id, "s0");
            }
            other => panic!("expected SetActiveScene, got {other:?}"),
        }
    }

    #[test]
    fn archived_layout_must_not_hold_active_scene() {
        let repairs = validate(&workspace(layout("archived", "s0")));
        assert_eq!(repairs.len(), 1);
        match &repairs[0] {
            Repair::SetActiveScene {
                class, scene_id, ..
            } => {
                assert_eq!(*class, RepairClass::Strong);
                assert_eq!(scene_id, "");
            }
            other => panic!("expected SetActiveScene, got {other:?}"),
        }
    }

    #[test]
    fn dangling_theme_ref_is_repaired_to_default() {
        let mut ws = workspace(layout("active", "s0"));
        ws.layouts[0].scenes[0].theme_id = "deleted-theme".into();
        let repairs = validate(&ws);
        let found = repairs.iter().find_map(|repair| match repair {
            Repair::SetSceneTheme {
                scene_id, theme_id, ..
            } => Some((scene_id.clone(), theme_id.clone())),
            _ => None,
        });
        assert_eq!(found, Some(("s0".into(), "cozy".into())));
    }

    #[test]
    fn registered_themes_need_no_repair() {
        // ADR-0007: built-ins and custom themes alike are registry entries.
        let mut ws = workspace(layout("active", "s0"));
        ws.layouts[0].scenes[0].theme_id = "cozy".into();
        ws.layouts[0].scenes[1].theme_id = "theme-custom".into();
        ws.themes = vec![
            ThemeDef {
                id: "cozy".into(),
                name: "Cozy".into(),
                base: String::new(),
                protected: true,
                tokens: Default::default(),
            },
            ThemeDef {
                id: "theme-custom".into(),
                name: "Custom".into(),
                base: "cozy".into(),
                protected: false,
                tokens: Default::default(),
            },
        ];
        assert!(validate(&ws).is_empty());
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
    fn apply_repairs_fixes_dangling_theme() {
        let doc = LoroDoc::new();
        build_default(&doc).unwrap();

        // Corrupt the live scene's theme to one that does not exist.
        let tree = doc.get_tree(schema::TREE);
        let live = tree.children(tree.roots()[0]).unwrap()[0];
        tree.get_meta(live)
            .unwrap()
            .insert("themeId", "deleted")
            .unwrap();
        doc.commit();

        let repairs = validate(&read_workspace(&doc));
        assert_eq!(repairs.len(), 1, "the dangling theme is detected");
        apply_repairs(&doc, &repairs).unwrap();

        let ws = read_workspace(&doc);
        assert_eq!(ws.layouts[0].scenes[0].theme_id, "cozy", "reset to default");
        assert!(validate(&ws).is_empty(), "no residual violations");
    }

    #[test]
    fn missing_active_layout_is_repaired_to_first() {
        // The active layout id names no existing layout (e.g. it was deleted).
        let ws = Workspace {
            active_layout_id: "ghost".into(),
            layouts: vec![layout("active", "s0")],
            themes: Vec::new(),
        };
        let found = validate(&ws).into_iter().find_map(|repair| match repair {
            Repair::SetActiveLayout { layout_id, .. } => Some(layout_id),
            _ => None,
        });
        assert_eq!(found, Some("L".into()), "reset to the surviving layout");
    }

    #[test]
    fn apply_repairs_fixes_missing_active_layout() {
        let doc = LoroDoc::new();
        build_default(&doc).unwrap();

        // Corrupt: point the workspace at a layout id that does not exist.
        doc.get_map(schema::WORKSPACE)
            .insert("activeLayoutId", "ghost")
            .unwrap();
        doc.commit();

        let repairs = validate(&read_workspace(&doc));
        assert!(
            repairs
                .iter()
                .any(|repair| matches!(repair, Repair::SetActiveLayout { .. })),
            "the missing active layout is detected"
        );
        apply_repairs(&doc, &repairs).unwrap();

        let ws = read_workspace(&doc);
        assert_eq!(
            ws.active_layout_id, ws.layouts[0].id,
            "repaired to the real layout"
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
