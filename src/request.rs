#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

use std::{clone, fmt};

use crate::{Error, HttpVer, Result, SPACE, URL_SAFE, Header, parse_headers};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
/// A parsed HTTP request
pub struct Request<'a, 'headers> {
    /// The HTTP request method. Either `Method::Get`, `Method::Post`, or `Method::Put`
    pub method: Method,
    /// The target URL for the request
    pub path: &'a str,
    /// The HTTP request headers
    pub headers: &'headers [crate::Header<'a>],
    /// The body of the request or an empty slice if there is no body
    pub body: &'a [u8],
}
impl<'a, 'headers> Request<'a, 'headers> {
  /// Construct a new Response from its parts
  /// Use an empty `&str` to create a `Respose` with no body
  #[inline]
  pub fn new(method: Method, path: &'a str, headers: &'headers [crate::Header<'a>], body: &'a [u8]) -> Self {
    Self {
      method,
      path,
      headers,
      body
    }
  }

  #[inline]
  /// The byte representation of the Request transmittible over wire
  pub fn as_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(format!("{} {} HTTP/1.1\r\n", self.method, self.path).as_bytes());
    for header in self.headers.iter() {
      if header.name.is_empty() {break;}
      bytes.extend(header.name.as_bytes());
      bytes.extend(b": ");
      bytes.extend(header.val);
      bytes.extend(b"\r\n");
    }
    bytes.extend(b"\r\n");
    bytes.extend(self.body);
    bytes
  }
   /// Parses the bytes of an HTTP request into a `Request`
   /// It parses headers into the `header_buf` you pass, if there is more headers than the length of the buffer you pass, an Err(Error::TooManyHeaders) is returned
  #[inline]
  pub fn parse(slice: &'a [u8], headers_buf: &'headers mut [crate::Header<'a>]) -> Result<Request<'a, 'headers>> {
    if slice.len() < 14 {return Err(Error::Malformed);}
    let mut offset = 0;
    let (method, read) = parse_method(slice)?;
    offset += read;
    let (path, read) = parse_path(&slice[offset..])?;
    offset += read;
    if slice[offset..].len() < 10 {return Err(Error::Malformed);}
    parse_http_version(&slice[offset..])?;
    offset += 10;
    let read = parse_headers(&slice[offset..], headers_buf)?;
    offset += read;
    Ok(Request::new(method, path, headers_buf, &slice[offset..]))
  }
}
impl<'a, 'headers> fmt::Display for Request<'a, 'headers> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let mut headers = String::new();
      for header in self.headers.iter() {
        if header.to_string().is_empty() {continue;} 
        headers.push_str(&format!("{}\r\n", header));
      }
      let body = match std::str::from_utf8(self.body) {
        Ok(v) => {
          v.to_string()
        },
        Err(_) => {
          format!("{:?}", self.body)
        },
      };
      f.write_str(format!("{} {} HTTP/1.1\r\n{}\r\n{}", self.method, self.path, headers, body).as_str())
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
/// The http method of a request. Only GET, POST, and PUT are supported
pub enum Method {
  /// The http GET method  
  Get,
  /// The http POST method
  Post,
  /// The http PUT method
  Put,
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
        };
        f.write_str(method)
    }
}

#[inline]
//parses the method and removes white space after it
fn parse_method(slice: &[u8]) -> Result<(Method, usize)> {
  if &slice[0..4] == b"GET " {
    return Ok((Method::Get, 4));
  } else if &slice[0..5] == b"POST " {
    return Ok((Method::Post, 5));
  } else if &slice[0..4] == b"PUT " {
    return Ok((Method::Put, 4));
  }
  Err(Error::Malformed)
}

#[inline]
// parses the path and removes the space after making sure it only contains URL safe characters
fn parse_path(slice: &[u8]) -> Result<(&str, usize)> {
  for (counter, character) in slice.iter().enumerate() {
    if URL_SAFE[*character as usize] {
      continue;
    } else if *character == SPACE {
      let path = &slice[..counter];
      if path.is_empty() {return Err(Error::Malformed);}
      //SAFETY: already checked that the input is valid ascii
      return Ok( (unsafe { std::str::from_utf8_unchecked(path) }, counter+1));
    }
    return Err(Error::Malformed);
  }
  Err(Error::Malformed)
}

#[inline]
//removes the \r\n after
fn parse_http_version(slice: &[u8]) -> Result<HttpVer> {
  if &slice[0..10] == b"HTTP/1.1\r\n" {
    Ok(HttpVer::One)
  } else if &slice[0..10] == b"HTTP/2.0\r\n" {
    Ok(HttpVer::Two)
  } else {Err(Error::Malformed)}
}


