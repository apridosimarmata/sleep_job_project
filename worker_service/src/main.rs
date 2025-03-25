mod infrastructure;
mod app;
mod domain;
mod config;

use std::sync::Arc;

use app::job::delivery::messaging::{JobMessagingHandler, JobMessagingHandlerImpl};
use app::job::usecase::job::JobWorkerUsecaseImpl;
use domain::repository::job::JobRepositoryImpl;
use domain::repository::repositories::{RepositoriesWrapper};
use domain::usecase::usecases::UsecasesWrapper;
use tokio::signal;
use infrastructure::messaging::messaging::Messaging;
use infrastructure::database::postgre;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configs: config::ConfigWrapper = config::init_config();

    // infras
    let db_pool = postgre::get_db_conn_pool(configs.postgre_config.url).await.unwrap_or_else(|err| panic!("Database connection error: {:?}", err));
    let messaging = Arc::new(Messaging::new("amqp://test:test@localhost:5672".to_string()).await);


    let repos = Arc::new(RepositoriesWrapper{
        job_repository: JobRepositoryImpl{
            conn:db_pool,
        },
    });

    let usecases = Arc::new(UsecasesWrapper {
        job_usecases: JobWorkerUsecaseImpl { repositories: repos.clone(),  messaging: messaging.clone()}
    });

    let job_handler = JobMessagingHandlerImpl::new(messaging.conn.clone(), Arc::new(usecases.job_usecases.clone()));
    let _ = job_handler.listen().await;

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    Ok(())
}


