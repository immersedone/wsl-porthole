//! WSL IP change watcher.
//!
//! 1. Subscribe to Hyper-V VmSwitch event log (Event ID 102)
//! 2. On event → wait 5s for IP to settle
//! 3. wsl hostname -I → compare to last known IP
//! 4. If changed → netsh reset + re-apply all rules + update firewall
//! 5. Toast notification to user

use wsl_porthole_core::ip::detect_wsl_ip;

pub async fn watch_loop(last_ip: &mut String) {
    // TODO: implement Windows event log subscription
    // For now, poll every 30 seconds as fallback
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        if let Ok(ip) = detect_wsl_ip() {
            if ip != *last_ip {
                tracing::info!("WSL IP changed: {} → {}", last_ip, ip);
                *last_ip = ip;
                // TODO: call netsh::reset_all() then re-apply rules
            }
        }
    }
}
