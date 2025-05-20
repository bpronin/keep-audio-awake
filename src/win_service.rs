use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::audio::KeepAwakeService;
use windows_service::service_control_handler::ServiceStatusHandle;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "KeepAudioAwakeService";

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        // Log error (for production code, use Windows Event Log)
        eprintln!("Service error: {e}");
    }
}

fn run_service() -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    let service_status_handle = register_service_ctrl_handler(Arc::clone(&running))?;

    update_service_status(&service_status_handle, ServiceState::Running)?;

    KeepAwakeService::run(running)?;

    update_service_status(&service_status_handle, ServiceState::Stopped)?;

    Ok(())
}

fn register_service_ctrl_handler(running: Arc<AtomicBool>) -> Result<ServiceStatusHandle, String> {
    service_control_handler::register(SERVICE_NAME, move |control_event| match control_event {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            running.store(false, Ordering::SeqCst);
            ServiceControlHandlerResult::NoError
        }
        _ => ServiceControlHandlerResult::NotImplemented,
    })
    .map_err(|e| format!("Failed to register service control handler: {e}"))
}

fn update_service_status(
    status_handle: &ServiceStatusHandle,
    current_state: ServiceState,
) -> Result<(), String> {
    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::ZERO,
            process_id: None,
        })
        .map_err(|e| format!("Failed to update service status: {e}"))
}

pub(crate) fn run_main() -> Result<(), String> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main).map_err(|e| e.to_string())?;
    
    Ok(())
}
