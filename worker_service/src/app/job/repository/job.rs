use common_lib::error::Err;
use sqlx::{Postgres, Transaction};

use crate::domain::{
    model::job_model::JobModel,
    repository::job::{JobRepository, JobRepositoryImpl},
};

impl JobRepository for JobRepositoryImpl {
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

    async fn update_job<'a>(&self, tx: &mut Transaction<'a, Postgres>, req: JobModel) -> Result<i64, Err> {
        print!("called {}", req.id);
        let row = sqlx::query!(
            "UPDATE ms_jobs SET
                n = $1,
                email = $2,
                created_at = $3,
                status = $4,
                finishes_at = $5
            WHERE id = $6", // Use RETURNING to get the inserted ID
            req.n,
            &req.email,
            req.created_at,
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

    async fn create_job<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>, // Mutable reference to Transaction
        req: JobModel,
    ) -> Result<i64, Err> {
        let row = sqlx::query!(
            "INSERT INTO ms_jobs (n, email, created_at, status, finishes_at) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id", // Use RETURNING to get the inserted ID
            req.n,
            &req.email,
            req.created_at,
            &req.status,
            req.finishes_at
        )
        .fetch_one(&mut **tx) // Use tx for transaction safety
        .await;

        match row {
            Ok(result) => Ok(result.id), // Return the inserted ID
            Err(err) => Err(Err {
                message: err.to_string(),
            }),
        }
    }
}
