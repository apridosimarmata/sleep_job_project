use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use actix_web::web::ServiceConfig;
use actix_web::web;
use actix_web_lab::sse::{self, Event };
use tokio::sync::mpsc::{channel, Receiver, Sender};
use crate::domain::dto::job_dto::JobRequestDTO;
use crate::domain::usecase::job::{JobUsecase, JobUsecaseImpl};
use crate::domain::usecase::usecases::UsecasesWrapper;
use tokio_stream::wrappers::ReceiverStream;




#[derive(Clone)] 
pub struct JobHTTPHandlerImpl {
    job_usecase: JobUsecaseImpl,
}
pub trait JobHTTPHandler {
    // async fn create_job(&self, job: web::Json<JobRequestDTO>) -> HTTPResponder<JobResponseDTO>;
    // async fn stream_jobs(&self) -> impl Responder;

}


pub fn register_job_routes(router_config: &mut ServiceConfig, usecases:  Arc<UsecasesWrapper>) {
    let job_http_handler = JobHTTPHandlerImpl {
        job_usecase: usecases.job_usecases.clone(),
    };

    router_config.app_data(web::Data::new(job_http_handler))
       .service(
            web::scope("/api/v1") 
                .route("/jobs", web::post().to(|data: web::Data<JobHTTPHandlerImpl>, job: web::Json<JobRequestDTO>| async move {
                    data.job_usecase.create_job(job.into_inner()).await
                }))
                .route("/jobs/stream", web::get().to(|data: web::Data<JobHTTPHandlerImpl>| async move {
                    let (tx, rx) : (Sender<Result<Event, Infallible>>, Receiver<Result<Event, Infallible>>) = channel(10);
            
                    data.job_usecase.stream_jobs(&tx).await;
            
                    // buffer size of 10
                    let data_stream: ReceiverStream<Result<Event, Infallible>> = ReceiverStream::new(rx);
                    
                    return sse::Sse::from_stream(data_stream).with_keep_alive(Duration::from_secs(5))
                }))
                ,
        );
}


impl JobHTTPHandler for JobHTTPHandlerImpl {
    // async fn create_job(&self, job: web::Json<JobRequestDTO>) -> HTTPResponder<JobResponseDTO> {
    //     self.job_usecase.create_job(job.into_inner()).await
    // }

}



