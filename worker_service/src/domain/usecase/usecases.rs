#[derive(Clone)] 
pub struct UsecasesWrapper {
    pub job_usecases: crate::app::job::usecase::job::JobWorkerUsecaseImpl,
}