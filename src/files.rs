//! Reading a file with a limit of size, writing one so that it is never left half written and locking one while it is changed. Every refusal starts with a code that does not
//! change (`file_too_large`, `file_unreadable`, `file_not_text`, `file_unwritable`, `file_locked`, `file_changed`), followed by the name of the file.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The file is larger than the limit (`max_file_mib` of `tools.cfg`).
pub const TOO_LARGE: &str = "file_too_large";
/// The file cannot be opened or read.
pub const UNREADABLE: &str = "file_unreadable";
/// The file is not text.
pub const NOT_TEXT: &str = "file_not_text";
/// The file cannot be written.
pub const UNWRITABLE: &str = "file_unwritable";
/// Another program is changing the file.
pub const LOCKED: &str = "file_locked";
/// The file is not the one that was read any more: another program changed it.
pub const CHANGED: &str = "file_changed";

/// The right to change a file, kept while the value lives. It is a lock that the system holds on the empty file `NAME.lock` beside it, and the system lets it go when the
/// program ends, however it ends: no lock is ever left behind to be cleared by hand. The `.lock` file itself stays and is never removed, because removing it would let two
/// programs hold two different files of the same name.
#[derive(Debug)]
pub struct EditLock {
    _held: std::fs::File,
}

/// Takes the right to change `file`, waiting up to `wait` for another program (or another part of this one) to let it go. Whoever reads a file, changes it and writes it
/// back holds the lock for the whole of it, so that no change is lost.
pub fn lock_for_edit(file: &Path, wait: Duration) -> Result<EditLock, String> {
    // the same file written another way (a relative path, a link) has the same lock
    let mut name = std::fs::canonicalize(file)
        .unwrap_or_else(|_| file.to_path_buf())
        .into_os_string();
    name.push(".lock");
    let lock = PathBuf::from(name);
    let fail = |e: std::io::Error| format!("{UNWRITABLE}: {}: {e}", lock.display());
    let held = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock)
        .map_err(fail)?;
    let start = Instant::now();
    loop {
        match held.try_lock() {
            Ok(()) => return Ok(EditLock { _held: held }),
            Err(std::fs::TryLockError::WouldBlock) => {}
            Err(std::fs::TryLockError::Error(e)) => return Err(fail(e)),
        }
        if start.elapsed() >= wait {
            return Err(format!(
                "{LOCKED}: {}: another program is changing this file; try again when it is done (lock_wait_s in tools.cfg says how long to wait)",
                file.display()
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

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

/// Writes `bytes` to a temporary file beside `file` (its name is unique to this process and call, so that two programs never share one), makes it durable on the disk and renames it
/// over `file`, so that a cut or a full disk leaves the old file whole and a power cut after the rename does not leave it empty. The temporary file is removed when anything fails.
/// Whether an existing file may be replaced is for the caller to decide before.
pub fn write_atomic(file: &Path, bytes: &[u8]) -> Result<(), String> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut tmp = file.as_os_str().to_owned();
    tmp.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let tmp = std::path::PathBuf::from(tmp);
    let fail = |e: std::io::Error| format!("{UNWRITABLE}: {}: {e}", file.display());
    let written = std::fs::File::create(&tmp).and_then(|mut f| {
        f.write_all(bytes)?;
        f.sync_all()
    });
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(fail(e));
    }
    std::fs::rename(&tmp, file).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        fail(e)
    })
}
