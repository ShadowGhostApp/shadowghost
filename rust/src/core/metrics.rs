use crate::core::types::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub active_connections: u32,
    pub message_count: u64,
    pub error_count: u32,
    pub uptime_seconds: u64,
    pub timestamp: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            active_connections: 0,
            message_count: 0,
            error_count: 0,
            uptime_seconds: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
}

pub struct MetricsCollector {
    config: Config,
    start_time: SystemTime,
    metrics_history: HashMap<String, Vec<MetricPoint>>,
    current_metrics: SystemMetrics,
    is_running: bool,
}

impl MetricsCollector {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            start_time: SystemTime::now(),
            metrics_history: HashMap::new(),
            current_metrics: SystemMetrics::default(),
            is_running: false,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        self.is_running = true;
        self.start_time = SystemTime::now();
        self.collect_metrics().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        self.is_running = false;
        Ok(())
    }

    pub async fn get_current_metrics(&self) -> Result<SystemMetrics, String> {
        Ok(self.current_metrics.clone())
    }

    pub fn get_metric_history(&self, metric_name: &str) -> Vec<MetricPoint> {
        self.metrics_history
            .get(metric_name)
            .cloned()
            .unwrap_or_default()
    }

    async fn collect_metrics(&mut self) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime = self
            .start_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        self.current_metrics = SystemMetrics {
            cpu_usage: self.get_cpu_usage(),
            memory_usage: self.get_memory_usage(),
            network_bytes_sent: self.current_metrics.network_bytes_sent,
            network_bytes_received: self.current_metrics.network_bytes_received,
            active_connections: self.current_metrics.active_connections,
            message_count: self.current_metrics.message_count,
            error_count: self.current_metrics.error_count,
            uptime_seconds: uptime,
            timestamp,
        };

        self.record_metric("cpu_usage", self.current_metrics.cpu_usage);
        self.record_metric("memory_usage", self.current_metrics.memory_usage as f64);
        self.record_metric("uptime_seconds", self.current_metrics.uptime_seconds as f64);

        Ok(())
    }

    fn record_metric(&mut self, name: &str, value: f64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let point = MetricPoint { timestamp, value };

        let history = self
            .metrics_history
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        history.push(point);

        if history.len() > 100 {
            history.remove(0);
        }
    }

    fn get_cpu_usage(&self) -> f64 {
        let load = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 100) as f64;
        load.min(100.0)
    }

    fn get_memory_usage(&self) -> u64 {
        1024 * 1024 * 10
    }

    pub fn increment_message_count(&mut self) {
        self.current_metrics.message_count += 1;
    }

    pub fn increment_error_count(&mut self) {
        self.current_metrics.error_count += 1;
    }

    pub fn update_network_stats(&mut self, bytes_sent: u64, bytes_received: u64) {
        self.current_metrics.network_bytes_sent += bytes_sent;
        self.current_metrics.network_bytes_received += bytes_received;
    }

    pub fn set_active_connections(&mut self, count: u32) {
        self.current_metrics.active_connections = count;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed().unwrap_or(Duration::from_secs(0))
    }

    pub fn reset_metrics(&mut self) {
        self.current_metrics = SystemMetrics::default();
        self.metrics_history.clear();
        self.start_time = SystemTime::now();
    }
}
