use actix_web::http::StatusCode;
use actix_web::{get, web, App, HttpResponse, HttpServer, ResponseError};
use serde::Serialize;

use crate::config::{Config, Logging};
use crate::etherscan::Etherscan;

mod config;
mod etherscan;

type BlockNumber = u64;
type BlockTime = u64;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::init();
    let _logging = Logging::init();

    log::info!("Starting eth-proxy server (version: {})", config.version());
    let address = config.address();
    HttpServer::new(move || {
        App::new()
            .data(Etherscan::new(config.etherscan_domain(), config.etherscan_api_key()))
            .service(current_block_time)
    })
    .bind(address)?
    .run()
    .await
}

#[get("/currentBlockTime")]
async fn current_block_time(context: web::Data<Etherscan>) -> HttpResponse {
    let etherscan = context.as_ref();
    match etherscan.current_block_number().await {
        Ok(current_block_number) => match etherscan.block_time(current_block_number).await {
            Ok(current_block_time) => {
                HttpResponse::Ok().json(CurrentBlockTime::new(current_block_number, current_block_time))
            }
            Err(failure) => HttpResponse::build(failure.code()).json(failure),
        },
        Err(failure) => HttpResponse::build(failure.code()).json(failure),
    }
}

#[derive(Serialize, Debug)]
pub struct CurrentBlockTime {
    block_number: BlockNumber,
    timestamp: BlockTime,
}

impl CurrentBlockTime {
    fn new(block_number: BlockNumber, timestamp: BlockTime) -> Self {
        CurrentBlockTime {
            block_number,
            timestamp,
        }
    }
}

#[derive(PartialEq, Serialize, Debug)]
struct InvocationFailure {
    #[serde(skip)]
    code: StatusCode,
    message: String,
}

impl InvocationFailure {
    fn failure(message: &str) -> Self {
        InvocationFailure {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }

    fn code(&self) -> StatusCode {
        self.code
    }
}

impl Into<InvocationFailure> for actix_web::client::SendRequestError {
    fn into(self) -> InvocationFailure {
        InvocationFailure {
            code: self.status_code(),
            message: self.to_string(),
        }
    }
}

impl Into<InvocationFailure> for actix_web::client::PayloadError {
    fn into(self) -> InvocationFailure {
        InvocationFailure {
            code: self.status_code(),
            message: self.to_string(),
        }
    }
}

impl Into<InvocationFailure> for serde_json::Error {
    fn into(self) -> InvocationFailure {
        InvocationFailure {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: self.to_string(),
        }
    }
}

impl Into<InvocationFailure> for std::num::ParseIntError {
    fn into(self) -> InvocationFailure {
        InvocationFailure {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: self.to_string(),
        }
    }
}
