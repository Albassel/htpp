
#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

//TODO: support http2, url parsing

//! # httpp
//!
//! A library for parsing HTTP requests and responses. Support for Http
//! 2.0 is planned for the future
//! 
//! The focus is on speed, safety, and security. It is intentionally designed to be strict to prevent a whole class of http attacks
//!


use core::{str, fmt};


#[cfg(test)]
mod tests;
/// Contains types and functions for parsing and generating http requests
pub mod request;
/// Contains types and functions for parsing and generating http responses
pub mod response;

pub use request::{Method, Request, RequestParser};
pub use response::{Response, ResponseParser};


const SPACE: u8 = 32;
const CR: u8 = 13;
const LF: u8 = 10;
const COLON: u8 = 58;
const HTAB: u8 = 9;
/// Makes it easier to create an empty header
pub const EMPTY_HEADER: Header = Header { name: "", val: b"" };
/// a result holding a parse error
pub type Result<T> = std::result::Result<T, Error>;
const URL_SAFE: [u8; 79] = [
    33,36,38,39,40,41,42,43,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,61,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,8,81,82,83,84,85,86,87,88,89,90,95,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122
];



#[derive(Debug, PartialEq, Eq)]
/// All parsing errors possible
/// Note that there is only one variant as to not give an attacker any information about why the parsing failed 
pub enum Error{
    /// The request is malformed and doesn't adhere to the standard
    Malformed
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("malformed request")
    }
}
impl std::error::Error for Error {}


#[derive(Debug)]
/// Represents an http header
pub struct Header<'a> {
    name: &'a str,
    val: &'a [u8],
}
impl<'a> fmt::Display for Header<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = match str::from_utf8(self.val) {
            Ok(v) => {
                format!("{}: {v}", self.name)
            },
            Err(_) => {
                format!("{}: {:?}", self.name, self.val)
            },
        };
        f.write_str(header.as_str())
    }
}
impl<'a> Header<'a> {
  fn new(name: &'a str, val: &'a [u8]) -> Self {
    Self {
        name,
        val
    }
  }
}


#[derive(Debug, PartialEq, Eq)]
/// Represents possible http versions
pub enum HttpVer {
    /// Http 1.1
    One,
    /// Http 2.0
    Two,
}
impl fmt::Display for HttpVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match self {
            Self::One => "HTTP/1.1",
            Self::Two => "HTTP/2.0"
        };
        f.write_str(version)
    }
}


