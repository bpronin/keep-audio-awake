use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows::core::PSTR;
use windows::Win32::Media::Audio::{
    waveOutClose, waveOutOpen, waveOutPrepareHeader, waveOutUnprepareHeader, waveOutWrite, CALLBACK_NULL, HWAVEOUT,
    WAVEFORMATEX, WAVEHDR, WAVE_FORMAT_PCM, WAVE_MAPPER,
};

pub(crate) struct KeepAwakeService {
    device_handler: HWAVEOUT,
    wave_header: WAVEHDR,
    _buffer: Vec<u8>, // Hold buffer to keep data alive while header uses it
}

impl KeepAwakeService {
    fn new() -> Result<Self, String> {
        const SAMPLES_PER_SEC: u32 = 44100;

        /* Open audio device */

        let audio_format = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM as u16,
            nChannels: 1,
            nSamplesPerSec: SAMPLES_PER_SEC,
            wBitsPerSample: 16,
            nBlockAlign: 2,
            nAvgBytesPerSec: SAMPLES_PER_SEC * 2,
            cbSize: 0,
        };

        let h_waveout = None;
        let result = unsafe {
            waveOutOpen(
                h_waveout,
                WAVE_MAPPER,
                &audio_format,
                Some(0),
                Some(0),
                CALLBACK_NULL,
            )
        };
        if result != 0 {
            return Err(format!("Error opening audio device ({})", result));
        }

        /* Create wave form with 100 ms of silence */

        let mut _buffer = vec![0; SAMPLES_PER_SEC as usize / 10]; // 0.1 second buffer
        let mut wave_header = WAVEHDR {
            lpData: PSTR(_buffer.as_mut_ptr()),
            dwBufferLength: (_buffer.len() * 2) as u32, 
            dwBytesRecorded: 0,
            dwUser: 0,
            dwFlags: 0,
            dwLoops: 0,
            lpNext: null_mut(),
            reserved: 0,
        };
        
        /* Prepare wave form for playback */
        unsafe {
            let device_handler = *h_waveout.unwrap();
            let result =
                waveOutPrepareHeader(device_handler, &mut wave_header, size_of::<WAVEHDR>() as u32);
            if result != 0 {
                waveOutClose(device_handler);
                return Err(format!("Error preparing audio header ({})", result));
            }

            Ok(Self {
                device_handler,
                wave_header,
                _buffer,
            })
        }
    }

    pub(crate) fn run(running: Arc<AtomicBool>) -> Result<(), String> {
        let mut service = Self::new()?;
        while running.load(Ordering::SeqCst) {
            service.play_silence()?;
            thread::sleep(Duration::from_secs(5));
        }

        Ok(())
    }

    fn play_silence(&mut self) -> Result<(), String> {
        let result = unsafe {
            waveOutWrite(
                self.device_handler,
                &mut self.wave_header,
                size_of::<WAVEHDR>() as u32,
            )
        };
        if result != 0 {
            Err(format!("Error writing audio data: {}", result))
        } else {
            Ok(())
        }
    }
}

impl Drop for KeepAwakeService {
    fn drop(&mut self) {
        unsafe {
            let result = waveOutUnprepareHeader(
                self.device_handler,
                &mut self.wave_header,
                size_of::<WAVEHDR>() as u32,
            );
            if result != 0 {
                eprintln!("Error unpreparing audio: {}", result);
            }
            waveOutClose(self.device_handler);
        }
        println!("Service stopped");
    }
}
