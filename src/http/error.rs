use reqwest::{
    Error as ReqwestError,
    header::InvalidHeaderValue,
    Response,
    StatusCode,
    Url,
};
use url::ParseError;

use std::{
    error::Error as StdError,
    fmt::{
        Display,
        Formatter,
        Result as FmtResult
    }
};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscordJsonError {
    pub code: isize,
    pub message: String,
    #[serde(skip)]
    non_exhaustive: (),
}

impl std::fmt::Debug for DiscordJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorResponse {
    pub status_code: StatusCode,
    pub url: Url,
    pub error: DiscordJsonError,
}

use async_trait::async_trait;
use crate::utils::AsyncFrom;

#[async_trait]
impl AsyncFrom<Response> for ErrorResponse {
    async fn async_from(r : Response) -> Self {
        ErrorResponse {
            status_code: r.status(),
            url: r.url().clone(),
            error: r.json().await.unwrap_or_else(|_| DiscordJsonError {
                code: -1,
                message: "[Serenity] No correct json was received!".to_string(),
                non_exhaustive: (),
            }),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    /// When a non-successful status code was received for a request.
    UnsuccessfulRequest(ErrorResponse),
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64` or `f64`.
    RateLimitI64F64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When parsing an URL failed due to invalid input.
    Url(ParseError),
    /// Header value contains invalid input.
    InvalidHeader(InvalidHeaderValue),
    /// Reqwest's Error contain information on why sending a request failed.
    Request(ReqwestError),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Error {
        Error::Request(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        Error::Url(error)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(error: InvalidHeaderValue) -> Error {
        Error::InvalidHeader(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { f.write_str(self.description()) }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::UnsuccessfulRequest(ref e) => &e.error.message,
            Error::RateLimitI64F64 => "Error decoding a header into an i64 or f64",
            Error::RateLimitUtf8 => "Error decoding a header from UTF-8",
            Error::Url(_) => "Provided URL is incorrect.",
            Error::InvalidHeader(_) => "Provided value is an invalid header value.",
            Error::Request(_) => "Error while sending HTTP request.",
            Error::__Nonexhaustive => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http_crate::response::Builder;

    #[test]
    fn test_error_response_into() {
        let error = DiscordJsonError {
            code: 43121215,
            message: String::from("This is a Ferris error"),
            non_exhaustive: (),
        };

        let mut builder = Builder::new();
        builder.status(403);
        let body = serde_json::to_string(&error).unwrap();
        let response = builder.body(body.into_bytes()).unwrap();

        let reqwest_response: reqwest::Response = response.into();
        let error_response: ErrorResponse = reqwest_response.into();

        let known = ErrorResponse {
            status_code: reqwest::StatusCode::from_u16(403).unwrap(),
            url: String::from("http://no.url.provided.local/").parse().unwrap(),
            error,
        };

        assert_eq!(error_response, known);
    }
}
