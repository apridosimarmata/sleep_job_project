mod infrastructure;
mod app;
mod domain;
mod config;

use std::sync::Arc;

use actix_web::{web::{self}, App, HttpServer};
use domain::repository::repositories;
use app::job::delivery::http::register_job_routes;
use infrastructure::messaging::messaging::{MessagingI, Messaging};
use infrastructure::database::postgre;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configs: config::ConfigWrapper = config::init_config();

    // infras
    let db_pool = postgre::get_db_conn_pool(configs.postgre_config.url).await.unwrap_or_else(|err| panic!("Database connection error: {:?}", err));
    let queues = Messaging::new("amqp://test:test@localhost:5672".to_string());

    let q =  queues.await;
    let _= q.create_exchange("logs_exchange").await;
    let _ = q.publish(&"Hallo kakkkk 1 ".to_string()).await.map_err(|err| print!("{}", err));
    // let _ = q.consume("queue").await;

    let _ = q.publish(&"Hallo kakkkk 2 ".to_string()).await.map_err(|err| print!("{}", err));
    let _ = q.publish(&"Hallo kakkkk 3 ".to_string()).await.map_err(|err| print!("{}", err));
    let _ = q.publish(&"Hallo kakkkk 4 ".to_string()).await.map_err(|err| print!("{}", err));
    let _ = q.publish(&"Hallo kakkkk 5 ".to_string()).await.map_err(|err| print!("{}", err));


    let repos = repositories::RepositoriesWrapper{
        job_repository: domain::repository::job::JobRepositoryImpl{
            conn:db_pool,
        },
    };

    let usecases = Arc::new(domain::usecase::usecases::UsecasesWrapper {
        job_usecases: domain::usecase::job::JobUsecaseImpl { repositories: repos }
    });


    let shared_data = web::Data::new(usecases.clone());

    HttpServer::new(move || {
        App::new()
        .app_data(shared_data.clone())
        .configure(|cfg| register_job_routes(cfg, usecases.clone())) // Correctly configures the App
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}


