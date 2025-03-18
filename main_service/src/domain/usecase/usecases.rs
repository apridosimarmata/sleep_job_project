#[derive(Clone)] 
pub struct UsecasesWrapper {
    pub job_usecases: crate::domain::usecase::job::JobUsecaseImpl,
}