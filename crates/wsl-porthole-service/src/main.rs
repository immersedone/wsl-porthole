//! Conduit Windows Service.
//! Watches for WSL IP changes (Hyper-V Event ID 102) and re-applies rules.

mod watcher;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Conduit service starting");
    // TODO: windows-service::service_dispatcher::start("Conduit", ffi_service_main)
    Ok(())
}
