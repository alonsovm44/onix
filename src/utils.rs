use std::path::PathBuf;

/// Returns the platform-specific root directory for the Onix toolset.
pub fn get_toolset_root() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\onix")
    } else {
        PathBuf::from("/mnt/bin/onix")
    }
}