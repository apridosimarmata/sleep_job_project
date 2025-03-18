use sqlx::{Pool, Postgres, Transaction};
use common_lib::error::Err;
use crate::domain::model::job_model::JobModel;

pub trait JobRepository {
    async fn get_tx(&self) -> Result<sqlx::PgTransaction, Err>;
    async fn create_job<'a>(&self, tx: &mut Transaction<'a, Postgres>, req: JobModel) -> Result<i64, Err>;
    async fn update_job<'a>(&self, tx: &mut Transaction<'a, Postgres>, req: JobModel) -> Result<i64, Err>;
}

#[derive(Clone)]
pub struct JobRepositoryImpl{
    pub conn: Pool<Postgres>,
}