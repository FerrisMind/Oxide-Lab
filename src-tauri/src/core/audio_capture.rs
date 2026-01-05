use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};
use tauri::{AppHandle, Emitter};

pub struct AudioCaptureState {
    capture: Mutex<Option<AudioCapture>>,
}

impl AudioCaptureState {
    pub fn new() -> Self {
        Self {
            capture: Mutex::new(None),
        }
    }

    pub fn start(&self, app: AppHandle) -> Result<(), String> {
        let mut guard = self
            .capture
            .lock()
            .map_err(|_| "Failed to lock audio capture state".to_string())?;
        if guard.is_some() {
            return Err("Recording already in progress".to_string());
        }
        let mut capture = AudioCapture::new(app)?;
        capture.start()?;
        *guard = Some(capture);
        Ok(())
    }

    pub fn stop(&self) -> Result<(Vec<f32>, u32), String> {
        let mut guard = self
            .capture
            .lock()
            .map_err(|_| "Failed to lock audio capture state".to_string())?;
        let capture = guard
            .take()
            .ok_or_else(|| "No recording in progress".to_string())?;
        Ok(capture.stop())
    }

    pub fn cancel(&self) -> Result<(), String> {
        let mut guard = self
            .capture
            .lock()
            .map_err(|_| "Failed to lock audio capture state".to_string())?;
        if guard.is_some() {
            guard.take();
        }
        Ok(())
    }
}

impl Default for AudioCaptureState {
    fn default() -> Self {
        Self::new()
    }
}

struct AudioCapture {
    device: Device,
    config: StreamConfig,
    sample_format: SampleFormat,
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<Stream>,
    rms_emitter: Option<Arc<Mutex<RmsEmitter>>>,
}

impl AudioCapture {
    fn new(app: AppHandle) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No input device available".to_string())?;
        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to query input config: {e}"))?;
        let rms_emitter = Arc::new(Mutex::new(RmsEmitter::new(app)));
        Ok(Self {
            device,
            config: config.clone().into(),
            sample_format: config.sample_format(),
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            rms_emitter: Some(rms_emitter),
        })
    }

    fn start(&mut self) -> Result<(), String> {
        let channels = self.config.channels;
        let buffer = Arc::clone(&self.buffer);
        let rms_emitter = self.rms_emitter.clone();
        let stream = match self.sample_format {
            SampleFormat::F32 => {
                build_stream::<f32>(&self.device, &self.config, channels, buffer, rms_emitter)?
            }
            SampleFormat::I16 => {
                build_stream::<i16>(&self.device, &self.config, channels, buffer, rms_emitter)?
            }
            SampleFormat::U16 => {
                build_stream::<u16>(&self.device, &self.config, channels, buffer, rms_emitter)?
            }
            _ => return Err("Unsupported audio sample format".to_string()),
        };
        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {e}"))?;
        self.stream = Some(stream);
        Ok(())
    }

    fn stop(mut self) -> (Vec<f32>, u32) {
        self.stream.take();
        let samples = self
            .buffer
            .lock()
            .map(|mut guard| std::mem::take(&mut *guard))
            .unwrap_or_default();
        (samples, self.config.sample_rate.0)
    }
}

fn build_stream<T: SizedSample + Send + Sync + 'static>(
    device: &Device,
    config: &StreamConfig,
    channels: u16,
    buffer: Arc<Mutex<Vec<f32>>>,
    rms_emitter: Option<Arc<Mutex<RmsEmitter>>>,
) -> Result<Stream, String>
where
    f32: FromSample<T>,
{
    let rms_emitter = rms_emitter.clone();
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &_| {
                // Determine if we have signal
                // (optional: log every N seconds?)
                let rms = push_input_data(data, channels, &buffer);
                if let (Some(rms), Some(emitter)) = (rms, rms_emitter.as_ref())
                    && let Ok(mut guard) = emitter.lock()
                {
                    guard.maybe_emit(rms);
                }
            },
            move |err: cpal::StreamError| log::error!("Audio input stream error: {err}"),
            None,
        )
        .map_err(|e| format!("Failed to build input stream: {e}"))
}

fn push_input_data<T: Sample>(
    data: &[T],
    channels: u16,
    buffer: &Arc<Mutex<Vec<f32>>>,
) -> Option<f32>
where
    f32: FromSample<T>,
{
    let channels = channels as usize;
    if channels == 0 {
        return None;
    }
    let mut mono = Vec::with_capacity(data.len() / channels);
    let mut sum_sq = 0.0f32;
    for frame in data.chunks(channels) {
        let mut sum = 0.0f32;
        for sample in frame {
            sum += sample.to_sample::<f32>();
        }
        let mono_sample = sum / channels as f32;
        mono.push(mono_sample);
        sum_sq += mono_sample * mono_sample;
    }
    if let Ok(mut guard) = buffer.lock() {
        guard.extend_from_slice(&mono);
    }
    if mono.is_empty() {
        return None;
    }
    Some((sum_sq / mono.len() as f32).sqrt())
}

pub fn resample_linear(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }
    if input_rate == output_rate {
        return input.to_vec();
    }
    let ratio = input_rate as f32 / output_rate as f32;
    let mut output_len = (input.len() as f32 / ratio).round() as usize;
    if output_len == 0 {
        output_len = 1;
    }
    let mut output = vec![0.0f32; output_len];
    for (i, value) in output.iter_mut().enumerate() {
        let position = i as f32 * ratio;
        let index = position.floor() as usize;
        let next = (index + 1).min(input.len().saturating_sub(1));
        let frac = position - index as f32;
        *value = (1.0 - frac) * input[index] + frac * input[next];
    }
    output
}

struct RmsEmitter {
    app: AppHandle,
    last_emit: Instant,
    interval: Duration,
}

impl RmsEmitter {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            last_emit: Instant::now(),
            interval: Duration::from_millis(80),
        }
    }

    fn maybe_emit(&mut self, rms: f32) {
        if self.last_emit.elapsed() < self.interval {
            return;
        }
        if let Err(err) = self.app.emit("voice_rms", rms) {
            log::error!("Failed to emit voice RMS: {err}");
        }
        self.last_emit = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::resample_linear;

    #[test]
    fn resample_linear_identity() {
        let input = vec![0.0, 0.5, -0.5, 1.0];
        let output = resample_linear(&input, 16_000, 16_000);
        assert_eq!(output, input);
    }

    #[test]
    fn resample_linear_downsample_length() {
        let input = vec![0.0; 48_000];
        let output = resample_linear(&input, 48_000, 16_000);
        assert_eq!(output.len(), 16_000);
    }

    #[test]
    fn resample_linear_upsample_length() {
        let input = vec![0.0; 16_000];
        let output = resample_linear(&input, 16_000, 48_000);
        assert_eq!(output.len(), 48_000);
    }
}
