extern crate ctor;

use crate::data::{CollectData, Data, DataType};
use crate::PDResult;
use crate::PERFORMANCE_DATA;
use chrono::prelude::*;
use ctor::ctor;
use log::debug;
use procfs::{CpuTime, KernelStats};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuData {
    pub time: DateTime<Utc>,
    pub cpu: i64,
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub irq: u64,
    pub softirq: u64,
    pub idle: u64,
    pub iowait: u64,
    pub steal: u64,
}

impl CpuData {
    fn new() -> Self {
        CpuData {
            time: Utc::now(),
            cpu: 0,
            user: 0,
            nice: 0,
            system: 0,
            irq: 0,
            softirq: 0,
            idle: 0,
            iowait: 0,
            steal: 0,
        }
    }

    fn set_data(&mut self, cpu: i64, time: DateTime<Utc>, cpu_time: &CpuTime) {
        self.cpu = cpu;
        self.time = time;
        self.user = cpu_time.user;
        self.nice = cpu_time.nice;
        self.system = cpu_time.system;
        self.irq = cpu_time.irq.unwrap_or_default();
        self.softirq = cpu_time.softirq.unwrap_or_default();
        self.idle = cpu_time.idle;
        self.iowait = cpu_time.iowait.unwrap_or_default();
        self.steal = cpu_time.steal.unwrap_or_default();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuUtilization {
    pub total: CpuData,
    pub per_cpu: Vec<CpuData>,
}

impl CpuUtilization {
    pub fn new() -> Self {
        CpuUtilization {
            total: CpuData::new(),
            per_cpu: Vec::<CpuData>::new(),
        }
    }

    fn set_total(&mut self, cpu: i64, time: DateTime<Utc>, total: CpuTime) {
        self.total.set_data(cpu, time, &total);
    }

    fn add_per_cpu_data(&mut self, cpu_data: CpuData) {
        self.per_cpu.push(cpu_data);
    }

    fn clear_per_cpu_data(&mut self) {
        self.per_cpu.clear();
    }
}

impl CollectData for CpuUtilization {
    fn collect_data(&mut self) -> PDResult {
        let stat = KernelStats::new().unwrap();
        let time_now = Utc::now();
        self.clear_per_cpu_data();

        /* Get total numbers */
        self.set_total(-1, time_now, stat.total);

        debug!("Total CPU Utilization: {:#?}", self.total);
        /* Get per_cpu numbers */
        for (i, cpu) in stat.cpu_time.iter().enumerate() {
            let mut current_cpu_data = CpuData::new();

            /* Set this CPU's data */
            current_cpu_data.set_data(i as i64, time_now, cpu);

            /* Push to Vec of per_cpu data */
            self.add_per_cpu_data(current_cpu_data);
        }
        debug!("Per CPU Utilization: {:#?}", self.per_cpu);
        Ok(())
    }
}

impl Default for CpuUtilization {
    fn default() -> Self {
        Self::new()
    }
}

#[ctor]
fn init_cpu_utilization() {
    let cpu_utilization = CpuUtilization::new();

    let dt = DataType {
        data: Data::CpuUtilization(cpu_utilization),
        file_handle: None,
        file_name: "cpu_utilization".to_string(),
        dir_name: String::new(),
        full_path: String::new(),
    };
    PERFORMANCE_DATA
        .lock()
        .unwrap()
        .add_datatype("CPU Utilization".to_string(), dt);
}

#[cfg(test)]
mod tests {
    use super::CpuUtilization;
    use crate::data::{CollectData, Data, DataType};

    #[test]
    fn test_init() {
        let cpu_utilization = DataType {
            data: Data::CpuUtilization(CpuUtilization::new()),
            file_handle: None,
            file_name: "cpu_utilization".to_string(),
            full_path: String::new(),
            dir_name: String::new(),
        };

        match cpu_utilization.data {
            Data::CpuUtilization(_) => assert!(true),
            _ => assert!(false, "CPU Utilization enum type error"),
        }
        assert!(cpu_utilization.file_handle.is_none());
        assert!(cpu_utilization.file_name == "cpu_utilization");
    }

    #[test]
    fn test_collect_data() {
        let mut cpu_utilization = CpuUtilization::new();

        assert!(cpu_utilization.collect_data().unwrap() == ());
        assert!(cpu_utilization.total.cpu == -1);
        assert!(!cpu_utilization.per_cpu.is_empty());
    }
}
