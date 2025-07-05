mod sensor;
mod task_scheduler;
use sensor::{SensorBuffer, SensorConfig, SensorType, run_sensor_loop};
use task_scheduler::{ScheduledTask, TaskPriority, run_scheduler};
use std::sync::Arc;
use tokio::task;
use tokio::sync::Mutex;
use log::info;
use log::warn;
use tokio::time::{sleep, Instant, Duration};

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Satellite Onboard Control System (OCS) Online.");

    // Correct structure: Arc-wrapped Tokio Mutex
    let shared_buffer = Arc::new(Mutex::new(SensorBuffer::new(100)));

    // Spawn sensor tasks
    let thermal = {
        let buffer_clone = shared_buffer.clone();
        task::spawn(run_sensor_loop(
            SensorConfig {
                sensor_type: SensorType::Thermal,
                interval_ms: 100,
                max_jitter_ms: 1,
            },
            buffer_clone,
        ))
    };

    let gyro = {
        let buffer_clone = shared_buffer.clone();
        task::spawn(run_sensor_loop(
            SensorConfig {
                sensor_type: SensorType::Gyro,
                interval_ms: 300,
                max_jitter_ms: 5,
            },
            buffer_clone,
        ))
    };

    let camera = {
        let buffer_clone = shared_buffer.clone();
        task::spawn(run_sensor_loop(
            SensorConfig {
                sensor_type: SensorType::Camera,
                interval_ms: 500,
                max_jitter_ms: 10,
            },
            buffer_clone,
        ))
    };

    let monitor_buffer = shared_buffer.clone();
    let thermal_monitor = task::spawn(async move {
    let mut missed_count = 0;

    loop {
        sleep(std::time::Duration::from_millis(100)).await;

        let now = Instant::now();
        let buf = monitor_buffer.lock().await;

        if let Some(last_time) = buf.last_thermal_time {
            let elapsed = now.duration_since(last_time).as_millis();

            if elapsed > 300 {
                missed_count += 1;
                if missed_count >= 3 {
                    warn!(
                        "[SAFETY ALERT] Thermal sensor missed {} cycles. Triggering safety protocol!",
                        missed_count
                    );
                }
            } else {
                missed_count = 0; // reset if normal
            }
        } else {
            missed_count += 1; // Never received any data
        }
    }
});


    info!("Sensors activated.");

    
    let scheduled_tasks = vec![
    ScheduledTask::new("Thermal Control", 100, TaskPriority::High, || {
        Box::pin(thermal_control())
    }),
    ScheduledTask::new("Health Monitoring", 250, TaskPriority::Medium, || {
        Box::pin(health_monitoring())
    }),
    ScheduledTask::new("Antenna Alignment", 500, TaskPriority::Low, || {
        Box::pin(antenna_alignment())
    }),
];

let running_task = Arc::new(Mutex::new(None::<(String, TaskPriority)>));
let scheduler_task = task::spawn(run_scheduler(scheduled_tasks, running_task));
    
    let _ = tokio::join!(thermal, gyro, camera,thermal_monitor,scheduler_task);

}

async fn thermal_control() {
    println!("↪ [Thermal Control] Regulating temperature...");
     tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

async fn health_monitoring() {
    println!("↪ [Health Monitor] Checking system status...");
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
}

async fn antenna_alignment() {
    println!("↪ [Antenna] Adjusting satellite orientation...");
    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
}

