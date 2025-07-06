use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

fn main() {
    let file = File::open("sensor_data.csv").expect("Failed to open CSV file");
    let reader = BufReader::new(file);

    let mut data: HashMap<String, Vec<f64>> = HashMap::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.trim().split(',').collect();

        if parts.len() >= 2 {
            let sensor_type = parts[0].to_string();
            let value: f64 = parts[1].parse().unwrap_or(0.0);

            data.entry(sensor_type).or_default().push(value);
        }
    }

    for (sensor, values) in data {
        let count = values.len();
        let avg = values.iter().sum::<f64>() / count as f64;
        let max = values.iter().cloned().fold(f64::MIN, f64::max);
        let min = values.iter().cloned().fold(f64::MAX, f64::min);

        println!(
            "{} | Count: {} | Avg: {:.3} | Min: {:.3} | Max: {:.3}",
            sensor, count, avg, min, max
        );
    }
}
