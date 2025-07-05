
use std::time::Duration;
use tokio::time::{sleep, Instant};
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::future::Future;
use std::pin::Pin;
// use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug)]
pub struct ScheduledTask {
    pub name: String,
    pub interval: Duration,
    pub priority: TaskPriority,
    pub last_run: Instant,
    pub logic: fn() -> Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl ScheduledTask {
    pub fn new(
        name: &str,
        interval_ms: u64,
        priority: TaskPriority,
        logic: fn() -> Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            interval: Duration::from_millis(interval_ms),
            priority,
            last_run: Instant::now(),
            logic,
        }
    }
}

pub async fn run_scheduler(
    mut tasks: Vec<ScheduledTask>,
    running_task: Arc<Mutex<Option<(String, TaskPriority)>>>, // Track name + priority
) {
    loop {
        let now = Instant::now();

        for task in tasks.iter_mut() {
            let elapsed = now.duration_since(task.last_run);

            if elapsed >= task.interval {
                let drift = elapsed.as_millis() as i64 - task.interval.as_millis() as i64;
                if drift > 5 {
                    warn!(
                        "[DRIFT] Task '{}' started {}ms late",
                        task.name, drift
                    );
                }

                // ⏱ Check for preemption
                {
                    let mut current = running_task.lock().await;

                    if let Some((ref running_name, ref running_priority)) = *current {
    if running_name != &task.name && task.priority > *running_priority {
        warn!(
            "[PREEMPT] '{}' preempting lower-priority task '{}'",
            task.name, running_name
        );
        *current = Some((task.name.clone(), task.priority.clone()));
    } else if running_name != &task.name {
        // Skip lower-priority task
        continue;
    }
} else {
    *current = Some((task.name.clone(), task.priority.clone()));
}


                    // Mark this task as currently running
                    *current = Some((task.name.clone(), task.priority.clone()));
                }

                let start = Instant::now();
                info!("[START] {}", task.name);

                // Await the task's logic properly
                (task.logic)().await;

                let duration = Instant::now().duration_since(start).as_millis();
                info!("[FINISH] {} ({}ms)", task.name, duration);

                // 🧹 Clear running task
                let mut current = running_task.lock().await;
                *current = None;

                task.last_run = Instant::now();

                if duration > task.interval.as_millis() {
                    warn!(
                        "[DEADLINE MISS] Task '{}' exceeded its interval ({}ms)",
                        task.name, duration
                    );
                }
            }
        }

        sleep(Duration::from_millis(10)).await;
    }
}
