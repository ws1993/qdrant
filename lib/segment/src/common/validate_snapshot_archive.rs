use std::fs::File;
use std::path::Path;

use tar::Archive;

use crate::common::operation_error::{OperationError, OperationResult};

pub fn open_snapshot_archive_with_validation(path: &Path) -> OperationResult<Archive<File>> {
    {
        let archive_file = File::open(path).map_err(|err| {
            OperationError::service_error(format!(
                "failed to open segment snapshot archive {path:?}: {err}"
            ))
        })?;
        let mut ar = Archive::new(archive_file);

        for entry in ar.entries_with_seek()? {
            // Read next archive entry type
            // Deliberately mask real error here for API users, it can expose arbitrary file contents
            let entry_type = match entry {
                Ok(entry) => entry.header().entry_type(),
                Err(err) => {
                    log::warn!("Error while reading snapshot archive, malformed entry: {err}");
                    return Err(OperationError::service_error(
                        "Failed to open snapshot archive, malformed format",
                    ));
                }
            };
            if !matches!(
                entry_type,
                tar::EntryType::Regular | tar::EntryType::Directory | tar::EntryType::GNUSparse
            ) {
                return Err(OperationError::ValidationError {
                    description: format!(
                        "Malformed snapshot, tar archive contains {entry_type:?} entry",
                    ),
                });
            }
        }
    }

    let archive_file = File::open(path).map_err(|err| {
        OperationError::service_error(format!(
            "failed to open segment snapshot archive {path:?}: {err}"
        ))
    })?;

    let mut ar = Archive::new(archive_file);
    ar.set_overwrite(false);
    ar.set_sync(true);

    Ok(ar)
}
