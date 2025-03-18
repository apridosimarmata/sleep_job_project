#[derive(Clone)]
pub struct RepositoriesWrapper {
    pub job_repository: crate::domain::repository::job::JobRepositoryImpl,
}