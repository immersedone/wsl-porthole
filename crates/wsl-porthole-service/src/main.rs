//! WSL PortHole Windows Service.
//!
//! Usage:
//!   wsl-porthole-service install   — install the Windows service
//!   wsl-porthole-service uninstall — remove the Windows service
//!   wsl-porthole-service run       — run as a Windows service (called by SCM)
//!   wsl-porthole-service standalone — run in foreground for debugging

mod ipc;
mod watcher;

use std::env;

const SERVICE_NAME: &str = "WslPortHole";
const SERVICE_DISPLAY: &str = "WSL PortHole";
const SERVICE_DESCRIPTION: &str =
    "Manages netsh portproxy rules between Windows, WSL2, and Docker. Auto-detects WSL IP changes.";

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("run");

    match command {
        "install" => install_service(),
        "uninstall" => uninstall_service(),
        "standalone" => run_standalone(),
        "run" => run_as_service(),
        "status" => print_status(),
        _ => {
            eprintln!("WSL PortHole Service v{}", env!("CARGO_PKG_VERSION"));
            eprintln!();
            eprintln!("Usage: wsl-porthole-service <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  install     Install as a Windows service");
            eprintln!("  uninstall   Remove the Windows service");
            eprintln!("  run         Run as a Windows service (called by SCM)");
            eprintln!("  standalone  Run in foreground (for debugging)");
            eprintln!("  status      Print current service status");
            Ok(())
        }
    }
}

/// Install the service into the Windows Service Control Manager.
fn install_service() -> anyhow::Result<()> {
    let exe_path = env::current_exe()?;
    println!("Installing {SERVICE_DISPLAY} service...");
    println!("  Binary: {}", exe_path.display());

    // Use sc.exe for portability (works without linking service manager APIs)
    let status = std::process::Command::new("sc")
        .args([
            "create",
            SERVICE_NAME,
            &format!("binPath= \"{}\" run", exe_path.display()),
            &format!("DisplayName= {SERVICE_DISPLAY}"),
            "start= auto",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("sc create failed. Are you running as Administrator?");
    }

    // Set description
    let _ = std::process::Command::new("sc")
        .args([
            "description",
            SERVICE_NAME,
            SERVICE_DESCRIPTION,
        ])
        .status();

    // Configure recovery: restart on first three failures
    let _ = std::process::Command::new("sc")
        .args([
            "failure",
            SERVICE_NAME,
            "reset= 86400",
            "actions= restart/5000/restart/10000/restart/30000",
        ])
        .status();

    println!("{SERVICE_DISPLAY} installed successfully.");
    println!("Start with: sc start {SERVICE_NAME}");
    Ok(())
}

/// Remove the service from the Windows Service Control Manager.
fn uninstall_service() -> anyhow::Result<()> {
    println!("Stopping {SERVICE_DISPLAY} service...");
    let _ = std::process::Command::new("sc")
        .args(["stop", SERVICE_NAME])
        .status();

    println!("Removing {SERVICE_DISPLAY} service...");
    let status = std::process::Command::new("sc")
        .args(["delete", SERVICE_NAME])
        .status()?;

    if !status.success() {
        anyhow::bail!("sc delete failed. Are you running as Administrator?");
    }

    println!("{SERVICE_DISPLAY} removed successfully.");
    Ok(())
}

/// Run in foreground (standalone mode for debugging).
fn run_standalone() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("{SERVICE_DISPLAY} starting in standalone mode");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        service_main_async().await;
    });
    Ok(())
}

/// Run as a Windows service via the service dispatcher.
///
/// On non-Windows platforms, falls back to standalone mode.
fn run_as_service() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        // The macro generates the proper extern "system" FFI entry point
        windows_service::define_windows_service!(ffi_service_main, service_entry);

        fn service_entry(arguments: Vec<std::ffi::OsString>) {
            if let Err(e) = run_service(arguments) {
                tracing::error!("Service failed: {e}");
            }
        }

        windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main)
            .map_err(|e| anyhow::anyhow!("Service dispatcher failed: {e}"))?;
    }

    #[cfg(not(windows))]
    {
        run_standalone()?;
    }

    Ok(())
}

/// Inner service logic for Windows service mode.
#[cfg(windows)]
fn run_service(_arguments: Vec<std::ffi::OsString>) -> anyhow::Result<()> {
    use std::time::Duration;
    use windows_service::service::*;
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    tracing_subscriber::fmt::init();
    tracing::info!("{SERVICE_DISPLAY} service starting");

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_tx = std::sync::Mutex::new(Some(shutdown_tx));

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                tracing::info!("Service stop requested");
                if let Ok(mut tx) = shutdown_tx.lock() {
                    if let Some(tx) = tx.take() {
                        let _ = tx.send(());
                    }
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Report running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        tokio::select! {
            _ = service_main_async() => {},
            _ = async { shutdown_rx.await.ok() } => {
                tracing::info!("Shutdown signal received");
            },
        }
    });

    // Report stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

/// Core async service logic shared between standalone and service modes.
async fn service_main_async() {
    tracing::info!("{SERVICE_DISPLAY} main loop starting");

    // Start IPC server for GUI communication
    let ipc_handle = tokio::spawn(async {
        if let Err(e) = ipc::run_ipc_server().await {
            tracing::error!("IPC server error: {e}");
        }
    });

    // Start the watcher loop
    let mut last_ip = String::new();
    if let Ok(ip) = wsl_porthole_core::ip::detect_wsl_ip() {
        tracing::info!("Initial WSL IP: {ip}");
        // Apply rules on first start
        if let Err(e) = watcher::apply_current_rules(&ip) {
            tracing::error!("Initial rule application failed: {e}");
        }
        last_ip = ip;
    }

    watcher::watch_loop(&mut last_ip).await;

    ipc_handle.abort();
}

/// Print current service status (query via sc).
fn print_status() -> anyhow::Result<()> {
    let output = std::process::Command::new("sc")
        .args(["query", SERVICE_NAME])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.is_empty() {
        println!("{SERVICE_DISPLAY} is not installed.");
    } else {
        print!("{stdout}");
    }
    Ok(())
}
