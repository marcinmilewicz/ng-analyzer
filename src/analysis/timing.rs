use std::time::Duration;

pub struct TimingMetrics {
    pub workspace_load_time: Duration,
    /// (project name, analysis duration) per processed project.
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

        let total_analysis_time: Duration = self
            .file_analysis_times
            .iter()
            .map(|(_, duration)| duration)
            .sum();

        println!("Total analysis time: {:?}", total_analysis_time);
        println!("Total execution time: {:?}", self.total_time);

        if !self.file_analysis_times.is_empty() {
            let avg_time = total_analysis_time / self.file_analysis_times.len() as u32;
            println!("Average analysis time per project: {:?}", avg_time);
        }
    }
}
