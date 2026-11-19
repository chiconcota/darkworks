use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use darkworks_schema::Tick;
use parking_lot::Mutex;
use crate::error::MediaResult;

pub struct AudioPlayer {
    _stream: Mutex<Option<cpal::Stream>>,
    is_playing: Arc<AtomicBool>,
    current_tick: Arc<AtomicU64>,
    sample_rate: u32,
    channels: u16,
    ring_buffer: Arc<Mutex<Vec<f32>>>,
}

impl AudioPlayer {
    pub fn try_new() -> MediaResult<Self> {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                log::warn!("Không tìm thấy thiết bị âm thanh cpal, chuyển sang chế độ giả lập đồng hồ.");
                return Ok(Self::dummy());
            }
        };

        let config = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Không lấy được cấu hình âm thanh cpal: {e}, dùng chế độ giả lập.");
                return Ok(Self::dummy());
            }
        };

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let is_playing = Arc::new(AtomicBool::new(false));
        let current_tick = Arc::new(AtomicU64::new(0));
        let ring_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(sample_rate as usize * 2)));

        let is_playing_cb = Arc::clone(&is_playing);
        let current_tick_cb = Arc::clone(&current_tick);
        let ring_buf_cb = Arc::clone(&ring_buffer);

        let err_fn = |err| log::error!("Lỗi cpal audio stream: {err}");

        let stream_result = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    write_samples_f32(data, &is_playing_cb, &current_tick_cb, &ring_buf_cb, sample_rate, channels);
                },
                err_fn,
                None,
            ),
            _ => {
                log::warn!("Định dạng mẫu âm thanh không phải f32, chuyển chế độ đồng hồ giả lập.");
                return Ok(Self::dummy());
            }
        };

        let stream = match stream_result {
            Ok(s) => {
                let _ = s.play();
                Some(s)
            }
            Err(e) => {
                log::warn!("Không thể build cpal output stream: {e}");
                None
            }
        };

        Ok(Self {
            _stream: Mutex::new(stream),
            is_playing,
            current_tick,
            sample_rate,
            channels,
            ring_buffer,
        })
    }

    pub fn dummy() -> Self {
        Self {
            _stream: Mutex::new(None),
            is_playing: Arc::new(AtomicBool::new(false)),
            current_tick: Arc::new(AtomicU64::new(0)),
            sample_rate: 48_000,
            channels: 2,
            ring_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn play(&self) {
        self.is_playing.store(true, Ordering::SeqCst);
    }

    pub fn pause(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn has_stream(&self) -> bool {
        self._stream.lock().is_some()
    }

    pub fn seek(&self, tick: Tick) {
        self.current_tick.store(tick.0, Ordering::SeqCst);
        self.ring_buffer.lock().clear();
    }

    pub fn current_tick(&self) -> Tick {
        Tick(self.current_tick.load(Ordering::SeqCst))
    }

    pub fn feed_samples(&self, samples: &[f32]) {
        let mut buf = self.ring_buffer.lock();
        let buf_len = buf.len();
        if buf_len + samples.len() > self.sample_rate as usize * 4 {
            // Giữ tối đa 2 giây buffer
            buf.drain(0..samples.len().min(buf_len));
        }
        buf.extend_from_slice(samples);
    }

    pub fn buffer_len(&self) -> usize {
        self.ring_buffer.lock().len()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

fn write_samples_f32(
    data: &mut [f32],
    is_playing: &Arc<AtomicBool>,
    current_tick: &Arc<AtomicU64>,
    ring_buf: &Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
) {
    if !is_playing.load(Ordering::Relaxed) {
        data.fill(0.0);
        return;
    }

    let mut buf = ring_buf.lock();
    let samples_needed = data.len();
    let available = buf.len();

    if available >= samples_needed {
        data.copy_from_slice(&buf[..samples_needed]);
        buf.drain(..samples_needed);
    } else {
        // Đệm thiếu: điền phần có sẵn và lấp 0
        data[..available].copy_from_slice(&buf);
        data[available..].fill(0.0);
        buf.clear();
    }

    // Cập nhật đồng hồ thời gian chuẩn (Tick Clock)
    let frames_played = (samples_needed / channels.max(1) as usize) as u64;
    let ticks_per_second = Tick::SECOND.0;
    let delta_ticks = (frames_played * ticks_per_second) / sample_rate.max(1) as u64;
    current_tick.fetch_add(delta_ticks, Ordering::Relaxed);
}
