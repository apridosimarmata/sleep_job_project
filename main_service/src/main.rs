mod infrastructure;
mod app;
mod domain;
mod config;

use std::sync::Arc;
use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

use actix_web::{web::{self}, App, HttpServer};
use domain::{misc::broadcast_channel::JobProggressBroadcaster, repository::repositories};
use app::job::delivery::{http::register_job_routes, messaging::{JobMessagingHandler, JobMessagingHandlerImpl}};
use infrastructure::messaging::messaging::{MessagingI, Messaging};
use infrastructure::database::postgre;
use common_lib::constants::*;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configs: config::ConfigWrapper = config::init_config();

    /* infras */
    let db = postgre::get_db_conn_pool(configs.postgre_config.url).await.unwrap_or_else(|err| panic!("Database connection error: {:?}", err));
    let messagging = Arc::new(Messaging::new(configs.rabbitmq_config.url).await);

    let _ = messagging.create_exchange(JOBS_EXCHANGE).await;
    let _ = messagging.create_queue(JOB_QUEUE, JOBS_EXCHANGE, JOB_TOPIC).await;
    let _ = messagging.create_queue(JOB_PROGRESS_QUEUE, JOBS_EXCHANGE, JOB_PROGRESS_TOPIC).await;


    let repos = repositories::RepositoriesWrapper{
        job_repository: domain::repository::job::JobRepositoryImpl{
            conn:db,
        },
    };

    let job_progress_channel: Arc<JobProggressBroadcaster> = Arc::new(JobProggressBroadcaster::new());
    let clone = job_progress_channel.clone();

    // tokio::spawn(async move {
    //    let mut new = job_progress_channel.rx.resubscribe();
    //    loop {
    //         match  new.recv().await {
    //             Ok(val) => println!("subscriber {:?}", val),
    //             Err(err) => println!("{}", err)
    //         }
            
    //    }
    // });

    let usecases = Arc::new(domain::usecase::usecases::UsecasesWrapper {
        job_usecases: domain::usecase::job::JobUsecaseImpl {
            repositories: repos,
            messaging: messagging.clone(),
            progress_channel: clone
        }
    });


    let job_handler = JobMessagingHandlerImpl::new(messagging.conn.clone(), usecases.clone());
    tokio::spawn(async move {
        let _ = job_handler.listen().await;
    });

    let shared_data = web::Data::new(usecases.clone());

    HttpServer::new(move || {
        App::new()
        .app_data(shared_data.clone())
        .configure(|cfg| register_job_routes(cfg, usecases.clone()))
        .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}


async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}