use sqlx::{Pool, Postgres, Transaction};
use common_lib::error::Err;
use crate::domain::model::job_model::{JobModel, UpdateJobStatusModel};

pub trait JobRepository {
    async fn get_tx(&self) -> Result<sqlx::PgTransaction, Err>;
    async fn update_job<'a>(&self, tx: &mut Transaction<'a, Postgres>, req: UpdateJobStatusModel) -> Result<i64, Err>;
    async fn get_job_by_id(&self, job_id: i64) -> Result<JobModel, Err>;
}

#[derive(Clone)]
pub struct JobRepositoryImpl{
    pub conn: Pool<Postgres>,
}