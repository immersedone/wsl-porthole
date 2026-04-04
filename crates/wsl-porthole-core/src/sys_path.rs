//! Full system paths for Windows executables.
//!
//! GUI apps launched via Explorer or shortcuts may not have System32 on their
//! PATH. We try multiple known locations and fall back to bare names.

use std::sync::OnceLock;

/// Find the first existing path, or fall back to the bare name.
fn find_exe(candidates: &[&str], fallback: &str) -> String {
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    fallback.to_string()
}

// Each executable is resolved once at first use and cached.

static WSL: OnceLock<String> = OnceLock::new();
pub fn wsl() -> &'static str {
    WSL.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\wsl.exe",
            r"C:\Windows\wsl.exe",
            r"C:\Windows\Sysnative\wsl.exe",
        ], "wsl.exe") }
        #[cfg(not(windows))]
        { "wsl".to_string() }
    })
}

static NETSH: OnceLock<String> = OnceLock::new();
pub fn netsh() -> &'static str {
    NETSH.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\netsh.exe",
            r"C:\Windows\Sysnative\netsh.exe",
        ], "netsh.exe") }
        #[cfg(not(windows))]
        { "netsh".to_string() }
    })
}

static POWERSHELL: OnceLock<String> = OnceLock::new();
pub fn powershell() -> &'static str {
    POWERSHELL.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            r"C:\Windows\Sysnative\WindowsPowerShell\v1.0\powershell.exe",
        ], "powershell.exe") }
        #[cfg(not(windows))]
        { "powershell".to_string() }
    })
}

static IPCONFIG: OnceLock<String> = OnceLock::new();
pub fn ipconfig() -> &'static str {
    IPCONFIG.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\ipconfig.exe",
            r"C:\Windows\Sysnative\ipconfig.exe",
        ], "ipconfig.exe") }
        #[cfg(not(windows))]
        { "ipconfig".to_string() }
    })
}

static NETSTAT: OnceLock<String> = OnceLock::new();
pub fn netstat() -> &'static str {
    NETSTAT.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\NETSTAT.EXE",
            r"C:\Windows\Sysnative\NETSTAT.EXE",
        ], "netstat.exe") }
        #[cfg(not(windows))]
        { "netstat".to_string() }
    })
}

static TASKLIST: OnceLock<String> = OnceLock::new();
pub fn tasklist() -> &'static str {
    TASKLIST.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\tasklist.exe",
            r"C:\Windows\Sysnative\tasklist.exe",
        ], "tasklist.exe") }
        #[cfg(not(windows))]
        { "tasklist".to_string() }
    })
}

static SC: OnceLock<String> = OnceLock::new();
pub fn sc() -> &'static str {
    SC.get_or_init(|| {
        #[cfg(windows)]
        { find_exe(&[
            r"C:\Windows\System32\sc.exe",
            r"C:\Windows\Sysnative\sc.exe",
        ], "sc.exe") }
        #[cfg(not(windows))]
        { "sc".to_string() }
    })
}
