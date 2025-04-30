use crate::ka_service::KeepAwakeService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
fn setup_exit_handler() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));

    let arc = Arc::clone(&running);
    ctrlc::set_handler(move || {
        println!("Stopping...");
        arc.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    println!("Press Ctrl+C to exit.");

    running
}

pub(crate) fn run_main() -> Result<(), String> {
    let running = setup_exit_handler();
    KeepAwakeService::run(running)?;
    
    Ok(())
}
