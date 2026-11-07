//! Reading a file with a limit of size and writing one so that it is never left half written. Every refusal starts with a code that does not change (`file_too_large`,
//! `file_unreadable`, `file_not_text`, `file_unwritable`), followed by the name of the file.

use std::io::Read;
use std::path::Path;

/// The file is larger than the limit (`max_file_mib` of `tools.cfg`).
pub const TOO_LARGE: &str = "file_too_large";
/// The file cannot be opened or read.
pub const UNREADABLE: &str = "file_unreadable";
/// The file is not text.
pub const NOT_TEXT: &str = "file_not_text";
/// The file cannot be written.
pub const UNWRITABLE: &str = "file_unwritable";

/// How many bytes `mib` MiB are.
pub fn mib(mib: u64) -> u64 {
    mib.saturating_mul(1_048_576)
}

/// The text of a file that is not larger than `max_bytes`. At most one byte more than the limit is ever read, also when the file grows while it is being read.
pub fn read_text_limited(path: &Path, max_bytes: u64) -> Result<String, String> {
    let shown = path.display();
    let file = std::fs::File::open(path).map_err(|e| format!("{UNREADABLE}: {shown}: {e}"))?;
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{UNREADABLE}: {shown}: {e}"))?;
    if bytes.len() as u64 > max_bytes {
        return Err(format!(
            "{TOO_LARGE}: {shown}: the file is larger than {max_bytes} bytes (max_file_mib in tools.cfg)"
        ));
    }
    String::from_utf8(bytes).map_err(|_| format!("{NOT_TEXT}: {shown}: the file is not text"))
}

/// Writes `bytes` to a temporary file beside `file` and renames it over `file`, so that a cut or a full disk leaves the old file whole. The temporary file is removed when
/// anything fails. Whether an existing file may be replaced is for the caller to decide before.
pub fn write_atomic(file: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    let fail = |e: std::io::Error| format!("{UNWRITABLE}: {}: {e}", file.display());
    if let Err(e) = std::fs::write(&tmp, bytes) {
        let _ = std::fs::remove_file(&tmp);
        return Err(fail(e));
    }
    std::fs::rename(&tmp, file).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        fail(e)
    })
}
