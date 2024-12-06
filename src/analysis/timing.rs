use std::time::Duration;

pub struct TimingMetrics {
    pub workspace_load_time: Duration,
    pub file_analysis_times: Vec<(String, Duration)>,
    pub total_time: Duration,
}

impl TimingMetrics {
    pub fn new() -> Self {
        TimingMetrics {
            workspace_load_time: Duration::default(),
            file_analysis_times: Vec::new(),
            total_time: Duration::default(),
        }
    }

    pub fn print_summary(&self) {
        println!("\n⏱️ Timing Analysis:");
        println!("Workspace load time: {:?}", self.workspace_load_time);

        let total_analysis_time: Duration = self.file_analysis_times.iter()
            .map(|(_, duration)| duration)
            .sum();

        println!("Total analysis time: {:?}", total_analysis_time);
        println!("Total execution time: {:?}", self.total_time);

        if !self.file_analysis_times.is_empty() {
            let avg_time = total_analysis_time / self.file_analysis_times.len() as u32;
            println!("Average analysis time per file: {:?}", avg_time);
        }
    }

    pub fn save_to_json(&self) -> Result<(), std::io::Error> {
        let timing_json = serde_json::json!({
            "workspace_load_time_ms": self.workspace_load_time.as_millis(),
            "total_time_ms": self.total_time.as_millis(),
            "file_analysis_times": self.file_analysis_times.iter()
                .map(|(path, duration)| (path.clone(), duration.as_millis()))
                .collect::<Vec<_>>()
        });

        std::fs::write(
            "timing-analysis.json",
            serde_json::to_string_pretty(&timing_json)?
        )
    }
}