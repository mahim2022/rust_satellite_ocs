use tokio::time::{Instant, Duration, sleep};
use std::collections::VecDeque;
use chrono::Utc;
// use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::warn;
use std::fs::OpenOptions;
use std::io::Write;
// use chrono::Utc; // Add to top of file

#[derive(Debug, Clone)]
pub enum SensorType {
    Thermal,
    Gyro,
    Camera,
}

#[derive(Debug, Clone)]
pub struct SensorData {
    pub sensor_type: SensorType,
    pub value: f64,
    pub timestamp: Instant,
    pub emitted_at: chrono::DateTime<chrono::Utc>,
}

// Configuration for sensor timing
pub struct SensorConfig {
    pub sensor_type: SensorType,
    pub interval_ms: u64,
    pub max_jitter_ms: u64,
}

// Shared buffer with max size
pub struct SensorBuffer {
    pub queue: VecDeque<SensorData>,
    pub max_size: usize,
    pub last_thermal_time: Option<Instant>,

}

impl SensorBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max_size: size,
            last_thermal_time: None,
        }
    }

    pub fn push(&mut self, data: SensorData) {
        if self.queue.len() >= self.max_size {
            warn!(
                "[DATA LOSS] Buffer full, dropped data from {:?} at {:?}",
                data.sensor_type, data.emitted_at
            );
        } else {
            if let SensorType::Thermal = data.sensor_type {
            self.last_thermal_time = Some(data.timestamp);
        }
            self.queue.push_back(data);
        }
    }
}

// Sensor loop
pub async fn run_sensor_loop(
    config: SensorConfig,
    buffer: Arc<Mutex<SensorBuffer>>,
) {
    let mut last_expected_time = Instant::now();

    loop {
        let now = Instant::now();
        let emitted_at = Utc::now();
        let expected_time = last_expected_time + Duration::from_millis(config.interval_ms);
        let jitter = now.duration_since(expected_time).as_millis() as i64;

        if jitter.abs() as u64 > config.max_jitter_ms {
            warn!(
                "[JITTER] {:?} sensor exceeded jitter: {}ms",
                config.sensor_type, jitter
            );
        }

        let data = SensorData {
            sensor_type: config.sensor_type.clone(),
            value: rand::random::<f64>(),
            timestamp: now,
            emitted_at,
        };

        // Insert into buffer
        // let arc_buf = buffer.lock().await;
        // let mut buf = arc_buf.lock().await;
        let mut buf = buffer.lock().await;


        buf.push(data);

        last_expected_time = expected_time;
        sleep(Duration::from_millis(config.interval_ms)).await;
    }
}
