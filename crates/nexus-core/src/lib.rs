//! `nexus-core`: the hexagonal core for Nexus.
//!
//! Holds the Loro-backed workspace document schema, plain read-model types, the
//! validator/repair logic, and the ports. It depends on `loro` for the CRDT
//! document but has no IO, async runtime, network, clock, or randomness (those
//! arrive via the `Clock`/`Random` ports). See ADR-0005.

#[cfg(test)]
mod smoke {
    //! Confirms the `loro` crate resolves and is usable. Real schema and
    //! validator tests land with their modules in the next step.

    #[test]
    fn loro_doc_constructs() {
        let _doc = loro::LoroDoc::new();
    }
}
