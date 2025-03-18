mod infrastructure;
mod app;
mod domain;
mod config;
use tokio::signal;

use infrastructure::messaging::messaging::{MessagingI, Messaging};
use infrastructure::database::postgre;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configs: config::ConfigWrapper = config::init_config();

    // infras
    let db_pool = postgre::get_db_conn_pool(configs.postgre_config.url).await.unwrap_or_else(|err| panic!("Database connection error: {:?}", err));
    let queues = Messaging::new("amqp://test:test@localhost:5672".to_string());

    let q =  queues.await;
    let _ = q.consume("queue").await;
    let _ = q.consume("queue_2").await;



    // let repos = repositories::RepositoriesWrapper{
    //     job_repository: domain::repository::job::JobRepositoryImpl{
    //         conn:db_pool,
    //     },
    // };

    // let usecases = Arc::new(domain::usecase::usecases::UsecasesWrapper {
    //     job_usecases: domain::usecase::job::JobWorkerUsecaseImpl { repositories: repos }
    // });

    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    Ok(())
}


