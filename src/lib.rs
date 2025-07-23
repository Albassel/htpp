
#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

//! # htpp
//!
//! A library for parsing HTTP requests and responses. The focus is on speed and safety. It is intentionally strict
//! to minimize HTTP attacks. It can also parse URLs
//! 
//! ## Working with [Request]
//! 
//! You can parse a request as follows:
//! 
//! ```rust
//! use htpp::{Request, EMPTY_HEADER};
//! 
//! let req = b"GET /index.html HTTP/1.1\r\n\r\n";
//! let mut headers = [EMPTY_HEADER; 10];
//! let parsed = Request::parse(req, &mut headers).unwrap();
//! assert!(parsed.method == htpp::Method::Get);
//! assert!(parsed.path == "/index.html");
//! ```
//! You can create a request as follows:
//! 
//! ```rust
//! use htpp::{Method, Request, Header};
//! 
//! let method = Method::Get;
//! let path = "/index.html";
//! let mut headers = [Header::new("Accept", b"*/*")];
//! let req = Request::new(method, path, &headers, b"");
//! ```
//! ## Working with [Response]
//! 
//! You can parse a response as follows:
//! 
//! ```rust
//! use htpp::{Response, EMPTY_HEADER};
//! 
//! let req = b"HTTP/1.1 200 OK\r\n\r\n";
//! let mut headers = [EMPTY_HEADER; 10];
//! let parsed = Response::parse(req, &mut headers).unwrap();
//! assert!(parsed.status == 200);
//! assert!(parsed.reason == "OK");
//! ```
//! 
//! You can create a response as follows:
//! 
//! ```rust
//! use htpp::{Response, Header};
//! 
//! let status = 200;
//! let reason = "OK";
//! let mut headers = [Header::new("Connection", b"keep-alive")];
//! let req = Response::new(status, reason, &mut headers, b"");
//! ```
//! 
//! After parsing a request, you can also parse the path part of the request inclusing query parameters as follows:
//! 
//! ```rust
//! use htpp::{Request, EMPTY_QUERY, Url, EMPTY_HEADER};
//! 
//! let req = b"GET /index.html?query1=value&query2=value HTTP/1.1\r\n\r\n";
//! let mut headers = [EMPTY_HEADER; 10];
//! let parsed_req = Request::parse(req, &mut headers).unwrap();
//! let mut queries_buf = [EMPTY_QUERY; 10];
//! let url = Url::parse(parsed_req.path.as_bytes(), &mut queries_buf).unwrap();
//! assert!(url.path == "/index.html");
//! assert!(url.query_params.unwrap()[0].name == "query1");
//! assert!(url.query_params.unwrap()[0].val == "value");
//! ```
//! 

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

use core::{str, fmt};

#[cfg(feature = "no_std")]
use alloc::format;

#[cfg(test)]
mod tests;
mod request;
mod response;
mod uri;

pub use request::{Method, Request};
pub use response::Response;
pub use uri::{Url, QueryParam, EMPTY_QUERY, UrlError};


const SPACE: u8 = 32;
const CR: u8 = 13;
const LF: u8 = 10;
const COLON: u8 = 58;
const HTAB: u8 = 9;
/// A result holding a parse error
pub type Result<T> = core::result::Result<T, Error>;

macro_rules! byte_map {
    ($($flag:expr,)*) => ([
        $($flag != 0,)*
    ])
}

static URL_SAFE: [bool; 256] = byte_map! [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//  \w !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  del
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
//   ====== Extended ASCII  ======
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static HEADER_NAME_SAFE: [bool; 256] = byte_map![
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//  \w !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  del
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];



#[derive(Debug, PartialEq, Eq)]
/// All parsing errors possible
pub enum Error{
    /// The request is malformed and doesn't adhere to the standard
    Malformed,
    /// The request has more headers than the length of the buffer you passed
    TooManyHeaders
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("malformed request")
    }
}
impl core::error::Error for Error {}


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


/// An empty header to make it easier to construct a header buffer to parse headers into
pub const EMPTY_HEADER: Header = Header{name: "", val: &[]};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// An HTTP header
pub struct Header<'a> {
    /// The name of the header
    pub name: &'a str,
    /// The value of the header
    pub val: &'a [u8],
}
impl<'a> fmt::Display for Header<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.name.is_empty() {
            let header = match str::from_utf8(self.val) {
                Ok(v) => {
                    format!("{}: {v}", self.name)
                },
                Err(_) => {
                    format!("{}: {:?}", self.name, self.val)
                },
            };
            return f.write_str(header.as_str());
        };
        f.write_str("")
    }
}
impl<'a> Header<'a> {
  /// Create a new HTTP header with the given name and value
  pub fn new(name: &'a str, val: &'a [u8]) -> Self {
    Self {
        name,
        val
    }
  }
}

#[inline]
// Parses the headers into the passed headers_buf
fn parse_headers<'a>(slice: &'a[u8], headers_buf: &mut [crate::Header<'a>]) -> Result<usize> {
  let mut offset = 0;
  let mut iteration = 0;
  while &slice[offset..(offset+2)] != b"\r\n" {
    if iteration >= headers_buf.len() {return Err(Error::TooManyHeaders);}
    let name = parse_header_name(&slice[offset..])?;
    offset += name.1;
    let val = parse_header_value(&slice[offset..])?;
    offset += val.1;
    headers_buf[iteration] = Header::new(name.0, val.0);
    iteration += 1;
  }
  Ok(offset+2)
}
#[inline]
// parses the header name and removes the `:` character and any spaces after it
fn parse_header_name(slice: &[u8]) -> Result<(&str, usize)> {
  for (counter, character) in slice.iter().enumerate() {
    if HEADER_NAME_SAFE[*character as usize] {
      continue;
    } else if *character == COLON {
      let name = &slice[..counter];
      if slice[counter+1] == SPACE || slice[counter+1] == 9 {
        //SAFETY: already checked that the input is valid ascii
        return Ok( (unsafe { core::str::from_utf8_unchecked(name) }, counter+2));
      }
      //SAFETY: already checked that the input is valid ascii
      return Ok( (unsafe { core::str::from_utf8_unchecked(name) }, counter+1));
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



