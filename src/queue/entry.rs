use std::collections::VecDeque;

use super::Job;

/// Output lines kept per job for the queue page.
const RECENT_LINES: usize = 6;

#[derive(Clone, Debug, PartialEq)]
pub enum JobState {
    Waiting,
    Running { progress: Option<f32> },
    Succeeded,
    Failed(String),
    Cancelled,
}

impl JobState {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Waiting | Self::Running { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JobEntry {
    pub job: Job,
    pub state: JobState,
    pub recent: VecDeque<String>,
}

impl JobEntry {
    pub fn new(job: Job) -> Self {
        Self {
            job,
            state: JobState::Waiting,
            recent: VecDeque::with_capacity(RECENT_LINES),
        }
    }

    pub fn push_line(&mut self, line: String) {
        if self.recent.len() == RECENT_LINES {
            self.recent.pop_front();
        }
        self.recent.push_back(line);
    }
}
