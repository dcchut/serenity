use reqwest::{header::InvalidHeaderValue, Error as ReqwestError, Response, StatusCode, Url};
use url::ParseError;

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
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

impl ErrorResponse {
    pub(crate) async fn async_from_response(r: Response) -> Self {
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
#[non_exhaustive]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::UnsuccessfulRequest(e) => f.write_str(&e.error.message),
            Error::RateLimitI64F64 => f.write_str("Error decoding a header into an i64 or f64"),
            Error::RateLimitUtf8 => f.write_str("Error decoding a header from UTF-8"),
            Error::Url(_) => f.write_str("Provided URL is incorrect."),
            Error::InvalidHeader(_) => f.write_str("Provided value is an invalid header value."),
            Error::Request(_) => f.write_str("Error while sending HTTP request."),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Url(inner) => Some(inner),
            Error::Request(inner) => Some(inner),
            _ => None,
        }
    }
}

//#[cfg(test)]
//mod test {
//    use super::*;
//    use http_crate::response::Builder;
//    use reqwest::ResponseBuilderExt;
//
//    #[test]
//    fn test_error_response_into() {
//        let error = DiscordJsonError {
//            code: 43121215,
//            message: String::from("This is a Ferris error"),
//            non_exhaustive: (),
//        };
//
//        let mut builder = Builder::new();
//        builder = builder.status(403);
//        builder = builder.url(String::from("https://ferris.crab").parse().unwrap());
//        let body_string = serde_json::to_string(&error).unwrap();
//        let response = builder.body(body_string.into_bytes()).unwrap();
//
//        let reqwest_response: reqwest::Response = response.into();
//        let error_response: ErrorResponse = ErrorResponse::async_from_response(reqwest_response).await;
//
//        let known = ErrorResponse {
//            status_code: reqwest::StatusCode::from_u16(403).unwrap(),
//            url: String::from("https://ferris.crab").parse().unwrap(),
//            error,
//        };
//
//        assert_eq!(error_response, known);
//    }
//}
