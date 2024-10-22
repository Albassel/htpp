
#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

//! # httpp
//!
//! A library for parsing HTTP requests and responses. The focus is on speed and safety. It is intentionally strict
//! to prevent possible HTTP attacks
//! 
//! ## Working with [Request]
//! 
//! You can parse a request as follows:
//! 
//! ```rust
//! use http::Request;
//! 
//! let req = b"GET /index.html HTTP/1.1\r\n\r\n";
//! let parsed = Request::parse(req).unwrap();
//! assert!(parsed.method() == htpp::Method::Get);
//! assert!(parsed.path() == "/index.html");
//! ```
//! You can create a request as follows:
//! 
//! ```rust
//! use http::{Method, Request, Header};
//! 
//! let method = Method::Get;
//! let path = "/index.html";
//! let headers = vec![Header::new("Accept", "*/*")];
//! let req = Request::new(method, path, headers);
//! ```
//! ## Working with [Response]
//! 
//! You can parse a response as follows:
//! 
//! ```rust
//! use http::Response;
//! 
//! let req = b"HTTP/1.1 200 OK GET\r\n\r\n";
//! let parsed = Response::parse(req).unwrap();
//! assert!(parsed.status() == 200);
//! assert!(parsed.reason() == "OK");
//! ```
//! 
//! You can create a response as follows:
//! 
//! ```rust
//! use http::{Response, Header};
//! 
//! let status = 200;
//! let reason = "OK";
//! let headers = vec![Header::new("Connection", "keep-alive")];
//! let req = Response::new(method, path, headers);
//! ```
//! 

use core::{str, fmt};


#[cfg(test)]
mod tests;
mod request;
mod response;

pub use request::{Method, Request};
pub use response::Response;


const SPACE: u8 = 32;
const CR: u8 = 13;
const LF: u8 = 10;
const COLON: u8 = 58;
const HTAB: u8 = 9;
/// A result holding a parse error
pub type Result<T> = std::result::Result<T, Error>;
const URL_SAFE: [u8; 79] = [
    33,36,38,39,40,41,42,43,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,61,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,95,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122
];
const REASON_PHRASE_SAFE: [u8; 53] = [
    32,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122
];


#[derive(Debug, PartialEq, Eq)]
/// All parsing errors possible
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


#[derive(Debug, PartialEq, Eq)]
/// Possible http versions
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



// ---------------------
// Parsing HTTP headers
// ---------------------



#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// An HTTP header
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

#[inline]
// Parses the headers with the empty line after them
fn parse_headers(slice: &[u8]) -> Result<(Vec<crate::Header>, usize)> {
  let mut headers = Vec::new();
  let mut offset = 0;
  while &slice[offset..(offset+2)] != b"\r\n" {
    let name = parse_header_name(&slice[offset..])?;
    offset += name.1;
    let val = parse_header_value(&slice[offset..])?;
    offset += val.1;
    headers.push(crate::Header::new(name.0, val.0));
  }
  Ok((headers, offset+2))
}
#[inline]
// parses the header name and removes the `:` character and any spaces after it
fn parse_header_name(slice: &[u8]) -> Result<(&str, usize)> {
  for (counter, character) in slice.iter().enumerate() {
    if (97..=122).contains(character) || (65..=90).contains(character) || *character == 45 {
      continue;
    } else if *character == COLON {
      let name = &slice[..counter];
      if slice[counter+1] == SPACE || slice[counter+1] == 9 {
        //SAFETY: already checked that the input is valid ascii
        return Ok( (unsafe { std::str::from_utf8_unchecked(name) }, counter+2));
      }
      //SAFETY: already checked that the input is valid ascii
      return Ok( (unsafe { std::str::from_utf8_unchecked(name) }, counter+1));
    }
    return Err(Error::Malformed);
  }
  unreachable!();
}
#[inline]
fn parse_header_value(slice: &[u8]) -> Result<(&[u8], usize)> {
  for (counter, character) in slice.iter().enumerate() {
    if *character == CR {
      let val = &slice[..counter];
      if slice[counter+1] == LF {
        return Ok((val, counter+2));
      }
      return Err(Error::Malformed);
    }
  }
  Err(Error::Malformed)
}



