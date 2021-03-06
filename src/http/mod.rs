//! The HTTP module which provides functions for performing requests to
//! endpoints in Discord's API.
//!
//! An important function of the REST API is ratelimiting. Requests to endpoints
//! are ratelimited to prevent spam, and once ratelimited Discord will stop
//! performing requests. The library implements protection to pre-emptively
//! ratelimit, to ensure that no wasted requests are made.
//!
//! The HTTP module comprises of two types of requests:
//!
//! - REST API requests, which require an authorization token;
//! - Other requests, which do not require an authorization token.
//!
//! The former require a [`Client`] to have logged in, while the latter may be
//! made regardless of any other usage of the library.
//!
//! If a request spuriously fails, it will be retried once.
//!
//! Note that you may want to perform requests through a [model]s'
//! instance methods where possible, as they each offer different
//! levels of a high-level interface to the HTTP module.
//!
//! [`Client`]: ../client/struct.Client.html
//! [model]: ../model/index.html

pub mod client;
pub mod error;
pub mod ratelimiting;
pub mod request;
pub mod routing;

pub use self::client::*;
pub use self::error::Error as HttpError;
pub use reqwest::StatusCode;

use self::request::Request;
use crate::model::prelude::*;
use reqwest::Method;
use std::{
    borrow::Cow,
    fs::File,
    path::{Path, PathBuf},
};

#[cfg(any(feature = "client", feature = "http"))]
use std::sync::Arc;

#[cfg(feature = "cache")]
use crate::cache::CacheRwLock;
#[cfg(feature = "client")]
use crate::client::Context;
#[cfg(feature = "client")]
use crate::CacheAndHttp;

/// This trait will be required by functions that need [`Http`] and can
/// optionally use a [`CacheRwLock`] to potentially avoid REST-requests.
///
/// The types [`Context`], [`CacheRwLock`], and [`Http`] implement this trait
/// and thus passing these to functions expecting `impl CacheHttp` is possible.
///
/// In a situation where you have the `cache`-feature enabled but you do not
/// pass a cache, the function will behave as if no `cache`-feature is active.
///
/// If you are calling a function that expects `impl CacheHttp` as argument
/// and you wish to utilise the `cache`-feature but you got no access to a
/// [`Context`], you can pass a tuple of `(CacheRwLock, Http)`.
///
/// [`CacheRwLock`]: ../cache/struct.CacheRwLock.html
/// [`Http`]: client/struct.Http.html
/// [`Context`]: ../client/struct.Context.html
pub trait CacheHttp {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http;
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        None
    }
}

#[cfg(feature = "client")]
impl CacheHttp for Context {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for &Context {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for &mut Context {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for &&mut Context {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for CacheAndHttp {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for &CacheAndHttp {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for Arc<CacheAndHttp> {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(feature = "client")]
impl CacheHttp for &Arc<CacheAndHttp> {
    #[cfg(feature = "http")]
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}

#[cfg(all(feature = "cache", feature = "http"))]
impl CacheHttp for (&CacheRwLock, &Http) {
    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.0)
    }
    fn http(&self) -> &Http {
        &self.1
    }
}

#[cfg(feature = "http")]
impl CacheHttp for &Http {
    fn http(&self) -> &Http {
        *self
    }
}

#[cfg(feature = "http")]
impl CacheHttp for Arc<Http> {
    fn http(&self) -> &Http {
        &*self
    }
}

#[cfg(feature = "http")]
impl CacheHttp for &Arc<Http> {
    fn http(&self) -> &Http {
        &*self
    }
}

#[cfg(all(feature = "cache", feature = "http"))]
impl AsRef<CacheRwLock> for (&CacheRwLock, &Http) {
    fn as_ref(&self) -> &CacheRwLock {
        self.0
    }
}

#[cfg(feature = "cache")]
impl AsRef<Http> for (&CacheRwLock, &Http) {
    fn as_ref(&self) -> &Http {
        self.1
    }
}

/// An method used for ratelimiting special routes.
///
/// This is needed because `reqwest`'s `Method` enum does not derive Copy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LightMethod {
    /// Indicates that a route is for the `DELETE` method only.
    Delete,
    /// Indicates that a route is for the `GET` method only.
    Get,
    /// Indicates that a route is for the `PATCH` method only.
    Patch,
    /// Indicates that a route is for the `POST` method only.
    Post,
    /// Indicates that a route is for the `PUT` method only.
    Put,
}

impl LightMethod {
    pub fn reqwest_method(self) -> Method {
        match self {
            LightMethod::Delete => Method::DELETE,
            LightMethod::Get => Method::GET,
            LightMethod::Patch => Method::PATCH,
            LightMethod::Post => Method::POST,
            LightMethod::Put => Method::PUT,
        }
    }
}

/// Enum that allows a user to pass a `Path` or a `File` type to `send_files`
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AttachmentType<'a> {
    /// Indicates that the `AttachmentType` is a byte slice with a filename.
    Bytes {
        data: Cow<'a, [u8]>,
        filename: String,
    },
    /// Indicates that the `AttachmentType` is a `File`
    File { file: &'a File, filename: String },
    /// Indicates that the `AttachmentType` is a `Path`
    Path(&'a Path),
    /// Indicates that the `AttachmentType` is an image URL.
    Image(&'a str),
}

impl<'a> From<(&'a [u8], &str)> for AttachmentType<'a> {
    fn from(params: (&'a [u8], &str)) -> AttachmentType<'a> {
        AttachmentType::Bytes {
            data: Cow::Borrowed(params.0),
            filename: params.1.to_string(),
        }
    }
}

impl<'a> From<&'a str> for AttachmentType<'a> {
    /// Constructs an `AttachmentType` from a string.
    /// This string may refer to the path of a file on disk, or the http url to an image on the internet.
    fn from(s: &'a str) -> AttachmentType<'_> {
        if s.starts_with("http://") || s.starts_with("https://") {
            AttachmentType::Image(s)
        } else {
            AttachmentType::Path(Path::new(s))
        }
    }
}

impl<'a> From<&'a Path> for AttachmentType<'a> {
    fn from(path: &'a Path) -> AttachmentType<'_> {
        AttachmentType::Path(path)
    }
}

impl<'a> From<&'a PathBuf> for AttachmentType<'a> {
    fn from(pathbuf: &'a PathBuf) -> AttachmentType<'_> {
        AttachmentType::Path(pathbuf.as_path())
    }
}

impl<'a> From<(&'a File, &str)> for AttachmentType<'a> {
    fn from(f: (&'a File, &str)) -> AttachmentType<'a> {
        AttachmentType::File {
            file: f.0,
            filename: f.1.to_string(),
        }
    }
}

/// Representation of the method of a query to send for the [`get_guilds`]
/// function.
///
/// [`get_guilds`]: fn.get_guilds.html
#[non_exhaustive]
pub enum GuildPagination {
    /// The Id to get the guilds after.
    After(GuildId),
    /// The Id to get the guilds before.
    Before(GuildId),
}

#[cfg(test)]
mod test {
    use super::AttachmentType;
    use std::path::Path;

    #[test]
    fn test_attachment_type() {
        assert!(
            match AttachmentType::from(Path::new("./dogs/corgis/kona.png")) {
                AttachmentType::Path(_) => true,
                _ => false,
            }
        );
        assert!(match AttachmentType::from("./cats/copycat.png") {
            AttachmentType::Path(_) => true,
            _ => false,
        });
    }
}
