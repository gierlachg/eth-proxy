use std::convert::TryInto;
use std::num::ParseIntError;

use actix_web::client::{Client, PayloadError};
use actix_web::web::Bytes;
use serde::Deserialize;

use crate::etherscan::EtherscanBlockReward::{Failure, Success};
use crate::{BlockNumber, BlockTime, InvocationFailure};

pub(super) struct Etherscan {
    domain: String,
    api_key: String,
    client: Client,
}

impl Etherscan {
    pub(super) fn new(domain: &str, api_key: &str) -> Self {
        Etherscan {
            domain: domain.to_string(),
            api_key: api_key.to_string(),
            client: Client::default(),
        }
    }

    pub(super) async fn current_block_number(&self) -> Result<BlockNumber, InvocationFailure> {
        self.get(&format!(
            "https://{}/api?module=proxy&action=eth_blockNumber&apikey={}",
            self.domain, self.api_key
        ))
        .await
        .and_then(|body| {
            serde_json::from_slice::<'_, EtherscanBlockNumber>(&body)
                .map_err(serde_json::Error::into)
                .and_then(|block_number| block_number.try_into())
        })
    }

    pub(super) async fn block_time(&self, block_number: BlockNumber) -> Result<BlockTime, InvocationFailure> {
        self.get(&format!(
            "https://{}/api?module=block&action=getblockreward&blockno={}&apikey={}",
            self.domain, block_number, self.api_key
        ))
        .await
        .and_then(|body| {
            serde_json::from_slice::<'_, EtherscanBlockReward>(&body)
                .map_err(serde_json::Error::into)
                .and_then(|block_reward| block_reward.try_into())
        })
    }

    async fn get(&self, url: &str) -> Result<Bytes, InvocationFailure> {
        match self.client.get(url).send().await {
            Ok(mut response) => response.body().await.map_err(PayloadError::into),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Deserialize)]
struct EtherscanBlockNumber<'a> {
    status: Option<&'a str>,
    result: &'a str,
}

impl<'a> TryInto<BlockNumber> for EtherscanBlockNumber<'a> {
    type Error = InvocationFailure;

    fn try_into(self) -> Result<BlockNumber, Self::Error> {
        if self.status.is_none() {
            BlockNumber::from_str_radix(self.result.trim_start_matches("0x"), 16).map_err(ParseIntError::into)
        } else {
            Err(InvocationFailure::failure(self.result))
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum EtherscanBlockReward<'a> {
    #[serde(borrow)]
    Success(EtherscanBlockRewardSuccess<'a>),
    #[serde(borrow)]
    Failure(EtherscanBlockRewardFailure<'a>),
}

#[derive(Deserialize, Debug)]
struct EtherscanBlockRewardSuccess<'a> {
    status: &'a str,
    message: &'a str,
    #[serde(borrow)]
    #[serde(rename(deserialize = "result"))]
    info: EtherscanBlockRewardInfo<'a>,
}

#[derive(Deserialize, Debug)]
struct EtherscanBlockRewardFailure<'a> {
    status: &'a str,
    result: &'a str,
}

#[derive(Deserialize, Debug)]
struct EtherscanBlockRewardInfo<'a> {
    #[serde(rename(deserialize = "timeStamp"))]
    timestamp: Option<&'a str>, // 'No record found' - racy or just buggy ?
}

impl<'a> TryInto<BlockTime> for EtherscanBlockReward<'a> {
    type Error = InvocationFailure;

    fn try_into(self) -> Result<BlockTime, Self::Error> {
        match self {
            Success(success) => {
                success
                    .info
                    .timestamp
                    .map_or(Err(InvocationFailure::failure(success.message)), |timestamp| {
                        timestamp
                            .parse::<BlockTime>()
                            .map_err(|e| InvocationFailure::failure(&e.to_string()))
                    })
            }
            Failure(failure) => Err(InvocationFailure::failure(failure.result)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_number_from_error() {
        let block_number = EtherscanBlockNumber {
            status: Some("0"),
            result: "Max rate limit reached",
        };
        let result: Result<BlockNumber, InvocationFailure> = block_number.try_into();
        assert_eq!(result, Err(InvocationFailure::failure("Max rate limit reached")));
    }

    #[test]
    fn test_block_number_from_success() {
        let block_number = EtherscanBlockNumber {
            status: None,
            result: "0x01ab",
        };
        assert_eq!(block_number.try_into(), Ok(427));
    }

    #[test]
    fn test_block_number_from_success_invalid_hex() {
        let block_number = EtherscanBlockNumber {
            status: None,
            result: "0xxyz",
        };
        let result: Result<BlockNumber, InvocationFailure> = block_number.try_into();
        assert_eq!(result, Err(InvocationFailure::failure("invalid digit found in string")));
    }

    #[test]
    fn test_block_time_from_error() {
        let block_time = EtherscanBlockReward::Failure(EtherscanBlockRewardFailure {
            status: "0",
            result: "Max rate limit reached",
        });
        let result: Result<BlockTime, InvocationFailure> = block_time.try_into();
        assert_eq!(result, Err(InvocationFailure::failure("Max rate limit reached")));
    }

    #[test]
    fn test_block_time_from_error_no_record_found() {
        let block_time = EtherscanBlockReward::Success(EtherscanBlockRewardSuccess {
            status: "0",
            message: "No record found",
            info: EtherscanBlockRewardInfo { timestamp: None },
        });
        let result: Result<BlockTime, InvocationFailure> = block_time.try_into();
        assert_eq!(result, Err(InvocationFailure::failure("No record found")));
    }

    #[test]
    fn test_block_time_from_success() {
        let block_time = EtherscanBlockReward::Success(EtherscanBlockRewardSuccess {
            status: "1",
            message: "OK",
            info: EtherscanBlockRewardInfo {
                timestamp: Some("123456789"),
            },
        });
        let result: Result<BlockTime, InvocationFailure> = block_time.try_into();
        assert_eq!(result, Ok(123456789));
    }

    #[test]
    fn test_block_time_from_success_invalid_timestamp() {
        let block_time = EtherscanBlockReward::Success(EtherscanBlockRewardSuccess {
            status: "1",
            message: "OK",
            info: EtherscanBlockRewardInfo {
                timestamp: Some("abcdefghij"),
            },
        });
        let result: Result<BlockTime, InvocationFailure> = block_time.try_into();
        assert_eq!(result, Err(InvocationFailure::failure("invalid digit found in string")));
    }
}
