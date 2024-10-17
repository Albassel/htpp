#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

use std::fmt;

use crate::{Error, HttpVer, Result, CR, LF, SPACE, URL_SAFE, COLON, Header};

/// A parsed http request
pub struct Request<'a> {
    method: Method,
    path: &'a str,
    headers: Vec<crate::Header<'a>>,
    body: &'a [u8],
}
impl<'a> Request<'a> {
  /// Construct a new Response from its parts
  /// Use an empty `&str` to create a `Respose` with no body
  #[inline]
  pub fn new(method: Method, path: &'a str, headers: Vec<crate::Header<'a>>, body: &'a [u8]) -> Self {
    Self {
      method,
      path,
      headers,
      body
    }
  }
  /// Returns the http method of the request
  pub fn method(&self) -> &Method {
    &self.method
  }
  /// Returns the target url of the request
  pub fn path(&self) -> &str {
    self.path
  }
  /// Returns the headers of the response
  pub fn headers(&self) -> &[Header<'a>] {
    &self.headers
  }
  /// returns the body of the request or an empty slice if there is no body
  pub fn body(&self) -> &[u8] {
    self.body
  }
  #[inline]
  /// The byte representation of the `Request` transmittible over wire
  pub fn as_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(format!("{} {} HTTP/1.1\r\n", self.method, self.path).as_bytes());
    for header in self.headers.iter() {
      bytes.extend(header.name.as_bytes());
      bytes.extend(b": ");
      bytes.extend(header.val);
      bytes.extend(b"\r\n");
    }
    bytes.extend(b"\r\n");
    bytes.extend(self.body);
    bytes
  }
}
impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let mut headers: String = self.headers.iter().map(|x| x.to_string() + "\r\n").collect();
      if headers.len() > 2 {
        headers.pop();
        headers.pop();
      }
      let body = match std::str::from_utf8(self.body) {
        Ok(v) => {
          v.to_string()
        },
        Err(_) => {
          format!("{:?}", self.body)
        },
      };
      f.write_str(format!("{} {} HTTP/1.1\r\n{}\r\n\r\n{}", self.method, self.path, headers, body).as_str())
    }
}


#[derive(Debug, PartialEq, Eq)]
/// The http method of a request. Only GET and POST are supported
pub enum Method {
  /// The http GET method  
  Get,
  /// The http POST method
  Post,
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match self {
            Self::Get => "GET",
            Self::Post => "POST"
        };
        f.write_str(method)
    }
}



/// A request parser that can parse the byte representation of a request
pub struct RequestParser<'a>{
    bytes: &'a [u8]
}

impl<'a> RequestParser<'a> {
  #[inline]
  /// Constructs a new `ResponseParser` that can parse the given byte slice
  pub fn new(content: &'a [u8]) -> Self {
    Self {bytes: content}
  }
  #[inline]
  // must only call if you know the buffer has >n characters
  fn advance(&mut self, n: usize) {
    self.bytes = &self.bytes[n..];
  }
  #[inline]
  fn find_substr(&self, substr: &'a [u8]) -> Option<usize> {
    self.bytes.windows(substr.len()).position(|win| win == substr)
  }
  #[inline]
  /// Parses the internal byte slice of the calling `RequestParser` returning `Ok(Request)` or an Err if the request is malformed
  pub fn parse(&mut self) -> Result<Request> {
    let idx = match self.find_substr(b"\r\n\r\n") {
      Some(idx) => idx,
      _ => {return Err(Error::Malformed);}
    };
    let (res, body) = self.bytes.split_at(idx+2);
    if res.len() < 14 {return Err(Error::Malformed);}
    self.bytes = res;
    let method = self.parse_method()?;
    let path = self.parse_path()?;
    if self.bytes.len() < 10 {return Err(Error::Malformed);}
    self.parse_http_version()?;
    let headers = self.parse_headers()?;
    Ok(Request{
      method,
      path,
      headers,
      body: &body[2..],
    })
  }
  #[inline]
  //parses the method and removes white space after it
  fn parse_method(&mut self) -> Result<Method> {
    if &self.bytes[0..4] == b"GET " {
      self.advance(4);
      return Ok(Method::Get);
    } else if &self.bytes[0..5] == b"POST " {
      self.advance(5);
      return Ok(Method::Get);
    } 
    Err(Error::Malformed)
  }
  #[inline]
  // parses the path and removes the space after making sure it only contains URL safe characters
  fn parse_path(&mut self) -> Result<&'a str> {
    for (counter, character) in self.bytes.iter().enumerate() {
      if URL_SAFE.binary_search(character).is_ok() {
        continue;
      } else if *character == SPACE {
        let (path, rest) = self.bytes.split_at(counter);
        self.bytes = &rest[1..];
        if path.is_empty() {return Err(Error::Malformed);}
        //SAFETY: already checked that the input is valid ascii
        return Ok( unsafe { std::str::from_utf8_unchecked(path) });
      }
      return Err(Error::Malformed);
    }
    Err(Error::Malformed)
  }
  #[inline]
  //removes the \r\n after
  fn parse_http_version(&mut self) -> Result<HttpVer> {
    if &self.bytes[0..10] == b"HTTP/1.1\r\n" {
      self.advance(10);
      Ok(HttpVer::One)
    } else if &self.bytes[0..10] == b"HTTP/2.0\r\n" {
      self.advance(10);
      Ok(HttpVer::Two)
    } else {Err(Error::Malformed)}
  }
  #[inline]
  fn parse_headers(&mut self) -> Result<Vec<crate::Header>> {
    let mut headers = Vec::new();
    while !self.bytes.is_empty() {
      let name = self.parse_header_name()?;
      let val = self.parse_header_value()?;
      headers.push(crate::Header {name, val});
    }
    Ok(headers)
  }
  #[inline]
  // parses the header name and removes the `:` character and any spaces after it
  fn parse_header_name(&mut self) -> Result<&'a str> {
    for (counter, character) in self.bytes.iter().enumerate() {
      // could use SIMD
      if (97..=122).contains(character) || (65..=90).contains(character) || *character == 45 {
        continue;
      } else if *character == COLON {
        let (name, rest) = self.bytes.split_at(counter);
        self.bytes = &rest[1..];
        if self.bytes[0] == SPACE || self.bytes[0] == 9 {
          self.advance(1);
        }
        //SAFETY: already checked that the input is valid ascii
        return Ok( unsafe { std::str::from_utf8_unchecked(name) });
      } else {
        return Err(Error::Malformed);
      }
    }
    Ok("")
  }
  #[inline]
  fn parse_header_value(&mut self) -> Result<&'a [u8]> {
    for (counter, character) in self.bytes.iter().enumerate() {
      if *character == CR {
        let (val, rest) = self.bytes.split_at(counter);
        self.bytes = &rest[1..];
        if self.bytes[0] == LF {
          self.advance(1);
          return Ok(val);
        }
        return Err(Error::Malformed);
      }
    }
    Err(Error::Malformed)
  }
}