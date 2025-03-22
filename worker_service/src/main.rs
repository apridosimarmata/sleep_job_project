mod infrastructure;
mod app;
mod domain;
mod config;

use std::sync::Arc;

use app::job::delivery::messaging::JobMessagingHandler;
use app::job::usecase::job::JobWorkerUsecaseImpl;
use domain::repository::repositories;
// use app::job::delivery::messaging::JobMessagingHandler;
use tokio::signal;
use infrastructure::messaging::messaging::Messaging;
use infrastructure::database::postgre;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configs: config::ConfigWrapper = config::init_config();

    // infras
    let db_pool = postgre::get_db_conn_pool(configs.postgre_config.url).await.unwrap_or_else(|err| panic!("Database connection error: {:?}", err));
    let messaging = Messaging::new("amqp://test:test@localhost:5672".to_string()).await;
    let messaging_conn = messaging.conn.lock().await;




    let repos = Arc::new(Mutex::new(repositories::RepositoriesWrapper{
        job_repository: domain::repository::job::JobRepositoryImpl{
            conn:db_pool,
        },
    }));

    let cloned_repos = repos.clone();
    let usecases = Arc::new(domain::usecase::usecases::UsecasesWrapper {
        job_usecases: JobWorkerUsecaseImpl { repositories: cloned_repos.clone() }
    });

    let job_handler = app::job::delivery::messaging::JobMessagingHandlerImpl::new(&messaging_conn, Arc::new(usecases.job_usecases.clone()));
    let _ = job_handler.listen().await;

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    Ok(())
}


