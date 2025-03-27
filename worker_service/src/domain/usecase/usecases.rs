use tokio::sync::Mutex;

use crate::app::job::usecase::job::JobWorkerUsecaseImpl;

pub struct UsecasesWrapper{
    pub job_usecases: Mutex<JobWorkerUsecaseImpl>,
}