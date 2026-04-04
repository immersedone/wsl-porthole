//! Full system paths for Windows executables.
//!
//! GUI apps launched via Explorer or shortcuts may not have System32 on their
//! PATH. Using absolute paths ensures all commands work regardless of how the
//! Tauri app was launched.

#[cfg(windows)]
pub fn wsl() -> &'static str { r"C:\Windows\System32\wsl.exe" }
#[cfg(not(windows))]
pub fn wsl() -> &'static str { "wsl" }

#[cfg(windows)]
pub fn netsh() -> &'static str { r"C:\Windows\System32\netsh.exe" }
#[cfg(not(windows))]
pub fn netsh() -> &'static str { "netsh" }

#[cfg(windows)]
pub fn powershell() -> &'static str { r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" }
#[cfg(not(windows))]
pub fn powershell() -> &'static str { "powershell" }

#[cfg(windows)]
pub fn ipconfig() -> &'static str { r"C:\Windows\System32\ipconfig.exe" }
#[cfg(not(windows))]
pub fn ipconfig() -> &'static str { "ipconfig" }

#[cfg(windows)]
pub fn netstat() -> &'static str { r"C:\Windows\System32\netstat.exe" }
#[cfg(not(windows))]
pub fn netstat() -> &'static str { "netstat" }

#[cfg(windows)]
pub fn tasklist() -> &'static str { r"C:\Windows\System32\tasklist.exe" }
#[cfg(not(windows))]
pub fn tasklist() -> &'static str { "tasklist" }

#[cfg(windows)]
pub fn sc() -> &'static str { r"C:\Windows\System32\sc.exe" }
#[cfg(not(windows))]
pub fn sc() -> &'static str { "sc" }
