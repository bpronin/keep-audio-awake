use crate::audio::keep_audio_awake;
use crate::util;
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
    util::check_app_running()?;

    let running = setup_exit_handler();
    keep_audio_awake(running)?;

    Ok(())
}
