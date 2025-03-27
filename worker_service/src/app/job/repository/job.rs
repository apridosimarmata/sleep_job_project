use common_lib::error::Err;
use sqlx::{Postgres, Transaction};

use crate::domain::{
    model::job_model::{JobModel, UpdateJobStatusModel},
    repository::job::{JobRepository, JobRepositoryImpl},
};

impl JobRepository for JobRepositoryImpl {

    async fn get_job_by_id(&self, job_id: i64) -> Result<JobModel, Err> {
        let row = sqlx::query_as!(
            JobModel,
            "SELECT id, n, email, created_at, status, finishes_at from ms_jobs where id = $1",
            job_id
        );
        let result = row.fetch_one(&self.conn).await;

        match result {
            Ok(job) => Ok(job),
            Err(e) => Err(Err{
                message:e.to_string()
            })
        }
    }

    async fn get_tx(&self) -> Result<sqlx::PgTransaction, Err> {
        self.conn.begin().await.map_or_else(
            |err| {
                Err(Err {
                    message: err.to_string(),
                })
            },
            |tx| Ok(tx),
        )
    }

    async fn update_job<'a>(&self, tx: &mut Transaction<'a, Postgres>, req: UpdateJobStatusModel) -> Result<i64, Err> {
        let row = sqlx::query!(
            "UPDATE ms_jobs SET
                status = $1,
                finishes_at = $2
            WHERE id = $3", // Use RETURNING to get the inserted ID
            &req.status,
            req.finishes_at,
            req.id
        )
        .execute(&mut **tx) // Use tx for transaction safety
        .await;

        match row {
            Ok(_) => Ok(req.id), // Return the inserted ID
            Err(err) => Err(Err {
                message: err.to_string(),
            }),
        }
    }
}
