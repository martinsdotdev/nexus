//! Atomic snapshot persistence for the canonical Loro document.
//!
//! Stores one binary Loro snapshot (`workspace.loro`) under a data directory.
//! Writes are atomic (temp file in the same directory + rename) so a crash mid
//! write never corrupts the snapshot; `tempfile::persist` handles the Windows
//! replace-existing case the plain rename would fail on.

use std::io::Write;
use std::path::{Path, PathBuf};

pub struct FilePersistence {
    path: PathBuf,
}

impl FilePersistence {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            path: data_dir.as_ref().join("workspace.loro"),
        }
    }

    /// Load the snapshot bytes, `None` on first run (no file yet).
    pub fn load(&self) -> std::io::Result<Option<Vec<u8>>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Atomically replace the snapshot.
    pub fn save(&self, snapshot: &[u8]) -> std::io::Result<()> {
        let dir = self
            .path
            .parent()
            .expect("snapshot path always has a parent directory");
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(snapshot)?;
        tmp.flush()?;
        tmp.persist(&self.path).map_err(|err| err.error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_load_is_none_then_round_trips_and_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let store = FilePersistence::new(dir.path());

        assert_eq!(store.load().unwrap(), None, "first run: no snapshot");

        store.save(&[1, 2, 3, 4]).unwrap();
        assert_eq!(store.load().unwrap(), Some(vec![1, 2, 3, 4]));

        store.save(&[9]).unwrap();
        assert_eq!(store.load().unwrap(), Some(vec![9]), "overwrite is atomic");
    }
}
