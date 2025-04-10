use tensorboard_proto::api::Status;

#[derive(Debug, Clone)]
pub enum JobStatus {
    Unknown,
    Success,
    Failure,
    Running,
}

impl From<JobStatus> for Status {
    fn from(value: JobStatus) -> Self {
        match value {
            JobStatus::Unknown => Status::STATUS_UNKNOWN,
            JobStatus::Success => Status::STATUS_SUCCESS,
            JobStatus::Failure => Status::STATUS_FAILURE,
            JobStatus::Running => Status::STATUS_RUNNING,
        }
    }
}
