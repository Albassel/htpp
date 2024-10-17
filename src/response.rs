
#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

use std::fmt;
use crate::{Error, HttpVer, Result, COLON, CR, LF, SPACE, Header};


/// A parsed http response
pub struct Response<'a> {
    status: u16,
    reason: &'a str,
    headers: Vec<Header<'a>>,
    body: &'a [u8],
}
impl<'a> Response<'a> {
  /// Construct a new Response from its parts.
  /// Use an empty `&str` to create a `Respose` with no reason phrase
  /// Use an empty `Vec` to create a `Respose` with no headers
  /// Use an empty `&str` to create a `Respose` with no body
  pub fn new(status: u16, reason: &'a str, headers: Vec<crate::Header<'a>>, body: &'a [u8]) -> Response<'a> {
    Self {
      status,
      reason,
      headers,
      body
    }
  }
  /// Returns the status of the response
  pub fn status(&self) -> u16 {
    self.status
  }
  /// Returns the reason phrase of the response or an empty `str` if there is no reason phrase
  pub fn reason(&self) -> &str {
    self.reason
  }
  /// Returns the headers of the response
  pub fn headers(&self) -> &[Header<'a>] {
    &self.headers
  }
  /// returns the body of the response or an empty slice if there is no body
  pub fn body(&self) -> &[u8] {
    self.body
  }
  /// The byte representation of the `Response` transmittible over wire
  pub fn bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    if self.reason.is_empty() {
      bytes.extend(format!("HTTP/1.1 {}\r\n", self.status).as_bytes());
    } else {
      bytes.extend(format!("HTTP/1.1 {} {}\r\n", self.status, self.reason).as_bytes());
    }
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
impl<'a> fmt::Display for Response<'a> {
    /// The string representation of the resonse
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let mut headers: String = self.headers.iter().map(|x| x.to_string() + "\r\n").collect();
      let body = match std::str::from_utf8(self.body) {
        Ok(v) => {
          v.to_string()
        },
        Err(_) => {
          format!("{:?}", self.body)
        },
      };
      if self.reason.is_empty() {
        f.write_str(format!("HTTP/1.1 {}\r\n{}\r\n{}", self.status, headers, body).as_str())
      } else {
        f.write_str(format!("HTTP/1.1 {} {}\r\n{}r\n{}", self.status, self.reason, headers, body).as_str())
      }
    }
}





/// A response parser that can parse the byte representation of a response
pub struct ResponseParser<'a>{
    bytes: &'a [u8]
}

impl<'a> ResponseParser<'a> {
  /// Constructs a new `ResponseParser` that can parse the given byte slice
  #[inline]
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
  /// Parses the internal byte slice of the calling `ResponseParser` returning `Ok(Response)` or an Err if the request is malformed
  pub fn parse(&mut self) -> Result<Response> {
    let idx = match self.find_substr(b"\r\n\r\n") {
      Some(idx) => idx,
      _ => {return Err(Error::Malformed);}
    };
    let (res, body) = self.bytes.split_at(idx+2);
    self.bytes = res;
    if res.len() < 12 {return Err(Error::Malformed);}
    self.parse_http_version()?;
    let (status, reason) = self.parse_status()?;
    let headers = self.parse_headers()?;
    Ok(Response{
      status,
      reason,
      headers,
      body: &body[2..],
    })
  }
  #[inline]
  //removes the space after
  fn parse_http_version(&mut self) -> Result<HttpVer> {
    if &self.bytes[0..9] == b"HTTP/1.1 " {
      self.advance(9);
      return Ok(HttpVer::One);
    } else if &self.bytes[0..9] == b"HTTP/2.0 " {
      self.advance(9);
      return Ok(HttpVer::Two);
    }
    Err(Error::Malformed)
  }
  #[inline]
  //parses the method and removes white space after it
  fn parse_status(&mut self) -> Result<(u16, &'a str)> {
    for (counter, character) in self.bytes.iter().enumerate() {
      // a number character
      if (48..=57).contains(character) {
        continue;
      } else if *character == SPACE {
        let (status, rest) = self.bytes.split_at(counter);
        self.bytes = &rest[1..];
        if status.len() > 3 {
          return Err(Error::Malformed);
        }
        //there is a reason phrase
        if (65..=90).contains(&self.bytes[0]) || (97..=122).contains(&self.bytes[0]) {
          let reason = self.parse_reason()?;
          //SAFETY: already checked that the input is valid ascii
          return Ok((str::parse::<u16>(unsafe {std::str::from_utf8_unchecked(status)}).unwrap(), reason));
          //there is no reason phrase
        } else if self.bytes[0] == CR {
          self.advance(1);
          if self.bytes[0] != LF {
            return Err(Error::Malformed);
          }
          self.advance(1);
          //SAFETY: already checked that the input is valid ascii
          return Ok((str::parse::<u16>(unsafe {std::str::from_utf8_unchecked(status)}).unwrap(), ""));
        } else {return Err(Error::Malformed);}
      } else if *character == CR {
        let (status, rest) = self.bytes.split_at(counter);
        self.bytes = &rest[1..];
        if self.bytes[0] != LF {
          return Err(Error::Malformed);
        }
        self.advance(1);
        //SAFETY: already checked that the input is valid ascii
        return Ok((str::parse::<u16>(unsafe {std::str::from_utf8_unchecked(status)}).unwrap(), ""));
      } else {
        return Err(Error::Malformed);
      }
    }
    Err(Error::Malformed)
  }

  #[inline]
  fn parse_reason(&mut self) -> Result<&'a str> {
    for (counter, character) in self.bytes.iter().enumerate() {
      if (65..=90).contains(character) || (97..=122).contains(character) || *character == SPACE {
        continue;
      } else if *character == CR {
        let (reason, rest) = self.bytes.split_at(counter);
        self.bytes = &rest[1..];
        if self.bytes[0] != LF {
          return Err(Error::Malformed);
        }
        self.advance(1);
        //SAFETY: already checked that the input is valid ascii
        return Ok( unsafe { std::str::from_utf8_unchecked(reason) });
      } else {
        return Err(Error::Malformed);
      }
    }
    Ok("")
  }
  #[inline]
  fn parse_headers(&mut self) -> Result<Vec<crate::Header>> {
    let mut headers = Vec::new();
    if let Some(&SPACE) = self.bytes.first() {return Err(Error::Malformed);}
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

