use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::System;

#[derive(Clone)]
pub struct SystemMonitor {
    sys: Arc<Mutex<System>>,
}

#[derive(Serialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub uptime: u64,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys: Arc::new(Mutex::new(sys)) }
    }

    pub fn get_metrics(&self) -> SystemMetrics {
        let mut sys = self.sys.lock().unwrap();

        // Refresh necessary components
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let uptime = System::uptime();

        SystemMetrics { cpu_usage, total_memory, used_memory, uptime }
    }
}
