/*
Copyright 2024 The Spice.ai OSS Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::fmt::Display;

mod datasets;
pub mod setup;

#[derive(Default)]
pub(crate) struct SearchBenchmarkResultBuilder {
    run_id: String,
    commit_sha: String,
    branch_name: String,

    config_name: String,

    started_at: i64,
    finished_at: i64,

    index_started_at: i64,
    index_finished_at: i64,

    search_started_at: i64,
    search_finished_at: i64,
    search_response_time: Vec<f64>,

    status: String,
}

impl Display for SearchBenchmarkResultBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "configuration: {},\n\
          status: {},\n\
          run_id: {},\n\
          commit_sha: {},\n\
          branch_name: {},\n\
          started_at: {},\n\
          index_time: {:.2} ms,\n\
          search_time: {:.2} ms,\n\
          rps: {:.2},\n\
          mean_response_time: {:.2} ms,\n\
          p95_response_time: {:.2} ms\n",
            self.config_name,
            self.status,
            self.run_id,
            self.commit_sha,
            self.branch_name,
            self.started_at,
            self.index_finished_at - self.index_started_at,
            self.search_finished_at - self.search_started_at,
            self.rps(),
            self.mean(),
            self.quantile(0.95)
        )
    }
}

impl SearchBenchmarkResultBuilder {
    pub fn new(
        commit_sha: impl Into<String>,
        branch_name: impl Into<String>,
        config_name: impl Into<String>,
    ) -> Self {
        Self {
            run_id: uuid::Uuid::new_v4().to_string(),
            commit_sha: commit_sha.into(),
            branch_name: branch_name.into(),
            config_name: config_name.into(),
            started_at: get_current_unix_ms(),
            ..Default::default()
        }
    }

    pub fn configuration_name(&self) -> String {
        self.config_name.clone()
    }

    pub fn start_index(&mut self) {
        self.index_started_at = get_current_unix_ms();
    }

    pub fn finish_index(&mut self) {
        self.index_finished_at = get_current_unix_ms();
    }

    pub fn start_search(&mut self) {
        self.search_started_at = get_current_unix_ms();
    }

    pub fn finish_search(&mut self) {
        self.search_finished_at = get_current_unix_ms();
    }

    pub fn record_response_time(&mut self, response_time: f64) {
        self.search_response_time.push(response_time);
    }

    pub fn finish(&mut self, is_successful: bool) {
        self.finished_at = get_current_unix_ms();
        self.status = if is_successful { "completed" } else { "failed" }.to_string();
    }

    // Calculate Requests Per Second (RPS)
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn rps(&self) -> f64 {
        let total_time_sec =
            (self.search_finished_at as f64 - self.search_started_at as f64) / 1000.0;
        self.search_response_time.len() as f64 / total_time_sec
    }

    // Calculate Mean Value
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn mean(&self) -> f64 {
        if self.search_response_time.is_empty() {
            0.0
        } else {
            self.search_response_time.iter().sum::<f64>() / self.search_response_time.len() as f64
        }
    }

    // Calculate Quantile (for example P95)
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    pub fn quantile(&self, quantile: f64) -> f64 {
        if self.search_response_time.is_empty() || !(0.0..=1.0).contains(&quantile) {
            return 0.0;
        }

        let mut sorted_times = self.search_response_time.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let idx = ((sorted_times.len() as f64) * quantile).ceil() as usize - 1;
        sorted_times[idx.clamp(0, sorted_times.len() - 1)]
    }
}

pub(crate) fn get_current_unix_ms() -> i64 {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}
