use std::sync::Arc;

use actix_web::web::ServiceConfig;
use actix_web::web;
use common_lib::http_response::HTTPResponder;
use crate::domain::dto::job_dto::{JobRequestDTO, JobResponseDTO};
use crate::domain::usecase::job::{JobWorkerUsecase, JobWorkerUsecaseImpl};
use crate::domain::usecase::usecases::UsecasesWrapper;

#[derive(Clone)] 
pub struct JobHTTPHandlerImpl {
    job_usecase: JobWorkerUsecaseImpl,
}
pub trait JobHTTPHandler {
    async fn create_job(&self, job: web::Json<JobRequestDTO>) -> HTTPResponder<JobResponseDTO>;
}


pub fn register_job_routes(router_config: &mut ServiceConfig, usecases:  Arc<UsecasesWrapper>) {
    let job_http_handler = JobHTTPHandlerImpl {
        job_usecase: usecases.job_usecases.clone(),
    };

    router_config.app_data(web::Data::new(job_http_handler))
       .service(
            web::scope("/api/v1") 
                .route("/jobs", web::post().to(|data: web::Data<JobHTTPHandlerImpl>, job: web::Json<JobRequestDTO>| async move {
                    data.create_job(job).await
                })),
        );
}


impl JobHTTPHandler for JobHTTPHandlerImpl {
    async fn create_job(&self, job: web::Json<JobRequestDTO>) -> HTTPResponder<JobResponseDTO> {
        self.job_usecase.create_job(job.into_inner()).await
    }
}



