use crate::util::{from_utf16, sleep_cancelable};
use std::ptr::null_mut;
use std::time::Duration;
use windows::core::PSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Media::Audio::{
    waveOutClose, waveOutGetErrorTextW, waveOutOpen, waveOutPrepareHeader, waveOutUnprepareHeader, waveOutWrite, CALLBACK_NULL,
    HWAVEOUT, WAVEFORMATEX, WAVEHDR, WAVE_FORMAT_PCM, WAVE_MAPPER,
    WHDR_DONE,
};
use windows::Win32::Media::MMSYSERR_NOERROR;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

pub const TIMER_AUDIO: usize = 100;
#[cfg(not(feature = "debug"))]
const TIMER_PERIOD_MS: u32 = 5000;
#[cfg(feature = "debug")]
const TIMER_PERIOD_MS: u32 = 2000;

const SAMPLES_PER_SEC: u32 = 44100;

#[derive(Default)]
pub struct AudioControl {
    window: Option<HWND>,
    device: HWAVEOUT,
    buffer: Vec<u8>,
    waveform: WAVEHDR,
}

impl AudioControl {
    pub fn start(&mut self, hwnd: Option<HWND>) -> Result<(), String> {
        self.window = hwnd;
        self.device = open_device()?;
        self.buffer = generate_waveform();
        self.waveform = create_waveform(&mut self.buffer);
        prepare_waveform(self.device, &mut self.waveform)?;

        unsafe {
            if SetTimer(self.window, TIMER_AUDIO, TIMER_PERIOD_MS, None) == 0 {
                Err("Failed to set timer")?
            }
        }

        Ok(())
    }

    pub fn play(&mut self) -> Result<(), String> {
        play_waveform(self.device, &mut self.waveform)?;
        await_play_done(&self.waveform);

        Ok(())
    }

    pub fn stop(&mut self) {
        unsafe {
            KillTimer(self.window, TIMER_AUDIO).unwrap_or_else(|e| {
                eprintln!("Failed to kill timer. {}", e);
            });
        }

        unprepare_waveform(self.device, &mut self.waveform);
        close_device(self.device);
    }
}

#[cfg(not(feature = "debug"))]
/// Generates 10 millis of silence.
fn generate_waveform() -> Vec<u8> {
    vec![0; SAMPLES_PER_SEC as usize / 100]
}

#[cfg(feature = "debug")]
/// Generates 1 second of 60Hz sine.
fn generate_waveform() -> Vec<u8> {
    let sample_count = SAMPLES_PER_SEC as usize;
    let amplitude = i16::MAX as f32;
    let frequency = 60.0;

    let mut buffer = Vec::with_capacity(sample_count * 2);
    for n in 0..sample_count {
        let t = n as f32 / SAMPLES_PER_SEC as f32;
        let sample = (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()) as i16;
        buffer.extend_from_slice(&sample.to_le_bytes());
    }

    buffer
}

fn create_waveform(buffer: &mut [u8]) -> WAVEHDR {
    WAVEHDR {
        lpData: PSTR(buffer.as_mut_ptr()),
        dwBufferLength: buffer.len() as u32,
        dwBytesRecorded: 0,
        dwUser: 0,
        dwFlags: 0,
        dwLoops: 0,
        lpNext: null_mut(),
        reserved: 0,
    }
}

macro_rules! win_api_call {
    ($expr:expr, $error_message:expr) => {{
        let result = unsafe { $expr };
        check_result(result, $error_message)
    }};
}

fn open_device() -> Result<HWAVEOUT, String> {
    let audio_format = WAVEFORMATEX {
        wFormatTag: WAVE_FORMAT_PCM as u16,
        nChannels: 1,
        nSamplesPerSec: SAMPLES_PER_SEC,
        wBitsPerSample: 16,
        nBlockAlign: 2,
        nAvgBytesPerSec: SAMPLES_PER_SEC * 2,
        cbSize: 0,
    };

    let mut handler = HWAVEOUT::default();

    win_api_call!(
        waveOutOpen(
            Some(&mut handler),
            WAVE_MAPPER,
            &audio_format,
            Some(0),
            Some(0),
            CALLBACK_NULL,
        ),
        "Error opening audio device"
    )?;

    Ok(handler)
}

fn close_device(device: HWAVEOUT) {
    win_api_call!(waveOutClose(device), "Error closing audio device").unwrap_or_else(|e| {
        eprintln!("{}", e);
    });
}

fn prepare_waveform(device: HWAVEOUT, waveform: &mut WAVEHDR) -> Result<(), String> {
    win_api_call!(
        waveOutPrepareHeader(device, waveform, size_of::<WAVEHDR>() as u32),
        "Error preparing waveform"
    )
}

fn unprepare_waveform(device: HWAVEOUT, waveform: &mut WAVEHDR) {
    win_api_call!(
        waveOutUnprepareHeader(device, waveform, size_of::<WAVEHDR>() as u32),
        "Error unpreparing waveform"
    )
    .unwrap_or_else(|e| {
        eprintln!("{}", e);
    });
}

fn play_waveform(device: HWAVEOUT, waveform: &mut WAVEHDR) -> Result<(), String> {
    win_api_call!(
        waveOutWrite(device, waveform, size_of::<WAVEHDR>() as u32),
        "Error playing waveform"
    )
}

fn await_play_done(waveform: &WAVEHDR) {
    /* wait for flag no more than 1 second.*/
    sleep_cancelable(Duration::from_secs(1), || {
        (waveform.dwFlags & WHDR_DONE) != 0
    });
}

fn check_result(result: u32, message: &str) -> Result<(), String> {
    if result == MMSYSERR_NOERROR {
        Ok(())
    } else {
        let error_text = unsafe {
            let mut text_buffer = [0u16; 256];
            let inner_result = waveOutGetErrorTextW(result, &mut text_buffer);
            if inner_result == MMSYSERR_NOERROR {
                from_utf16(&text_buffer)
            } else {
                format!("Error getting error text (code: {})", inner_result)
            }
        };
        Err(format!("{} (code: {}). {}", message, result, error_text))
    }
}

// pub fn keep_audio_awake(running: Arc<AtomicBool>, event_sink: Sender<u8>) -> Result<(), String> {
//     let device = open_device()?;
//     let mut buffer = generate_waveform();
//     let mut waveform = create_waveform(&mut buffer);
//     prepare_waveform(device, &mut waveform)?;
//
//     while running.load(Ordering::SeqCst) {
//         play_waveform(device, &mut waveform)?;
//         await_play_done(&waveform);
//
//         sleep_cancelable(Duration::from_millis(TIMER_PLAY_PERIOD_MS as u64), || {
//             !running.load(Ordering::Relaxed)
//         });
//     }
//
//     unprepare_waveform(device, &mut waveform);
//     close_device(device);
//
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use crate::audio::{
        await_play_done, check_result, close_device, create_waveform, generate_waveform,
        open_device, play_waveform, prepare_waveform, unprepare_waveform,
    };
    use windows::Win32::Media::{MMSYSERR_INVALPARAM, MMSYSERR_NOERROR};

    #[test]
    fn test_check_result() {
        assert!(check_result(MMSYSERR_NOERROR, "Error message").is_ok());
        assert!(check_result(MMSYSERR_INVALPARAM, "Error message").is_err())
    }

    #[test]
    fn test_generate_waveform() {
        let buffer = generate_waveform();
        assert_ne!(0, buffer.len());
    }

    #[test]
    fn test_create_audio() {
        let mut buffer = generate_waveform();
        let waveform = create_waveform(&mut buffer);
        assert_ne!(0, waveform.dwBufferLength as u32);
    }

    #[test]
    fn test_open_close_device() {
        let device = open_device().unwrap();
        close_device(device);
    }

    #[test]
    fn test_play_waveform() {
        let device = open_device().unwrap();
        let mut buffer = generate_waveform();
        let mut waveform = create_waveform(&mut buffer);

        prepare_waveform(device, &mut waveform).unwrap();
        play_waveform(device, &mut waveform).unwrap();
        await_play_done(&waveform);
        unprepare_waveform(device, &mut waveform);
        close_device(device);
    }
}
