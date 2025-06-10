#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides console window */
use log::error;

mod audio;
mod gui;
mod util;

fn setup_logger() {
    use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};

    Logger::try_with_str("debug")
        .expect("Failed to create logger")
        .log_to_file(FileSpec::default().directory(".log"))
        .set_palette("1;3;4;2;7".into())
        .rotate(
            Criterion::Size(10_000_000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .duplicate_to_stdout(Duplicate::All)
        .start()
        .expect("Failed to initialize logger.");

    std::panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
    }));
}

fn main() -> Result<(), String> {
    setup_logger();
    gui::run_main()?;

    Ok(())
}
