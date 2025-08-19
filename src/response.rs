
#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

use core::fmt;
use crate::{Error, HttpVer, Result, CR, LF, SPACE, Header, parse_headers, HEADER_NAME_SAFE};

#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(feature = "no_std")]
use alloc::format;
#[cfg(feature = "no_std")]
use alloc::string::ToString;


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
/// A parsed http response
pub struct Response<'a, 'headers> {
    /// The status code of the response
    pub status: u16,
    /// The reason phrase of the response or an empty string if it doesn't exist
    pub reason: &'a str,
    /// The HTTP response headers
    pub headers: &'headers [Header<'a>],
    /// The body of the response or an empty slice if there is no body
    pub body: &'a [u8],
}
impl<'a, 'headers> Response<'a, 'headers> {
  /// Construct a new `Response` from its parts.
  /// Use an empty `&str` to create a `Respose` with no reason phrase
  /// Use an empty `&str` to create a `Respose` with no body
  pub fn new(status: u16, reason: &'a str, headers: &'headers [Header<'a>], body: &'a [u8]) -> Response<'a, 'headers> {
    Self {
      status,
      reason,
      headers,
      body
    }
  }
  /// The byte representation of the `Response` transmittible over wire
  #[inline]
  pub fn as_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(128 + self.body.len());

    bytes.extend_from_slice(b"HTTP/1.1 ");
    bytes.extend_from_slice(self.status.to_string().as_bytes());
    if !self.reason.is_empty() {
        bytes.push(SPACE);
        bytes.extend_from_slice(self.reason.as_bytes());
    }
    bytes.extend_from_slice(b"\r\n");

    for header in self.headers.iter() {
      if header.name.is_empty() {break;}
      bytes.extend_from_slice(header.name.as_bytes());
      bytes.extend_from_slice(b": ");
      bytes.extend_from_slice(header.val);
      bytes.extend_from_slice(b"\r\n");
    }
    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(self.body);
    bytes
  }
  /// Parses the bytes of an HTTP response into a `Response`
  /// It parses headers into the `header_buf` you pass, if there is more headers than the length of the buffer you pass, an Err(Error::TooManyHeaders) is returned
  #[inline]
  pub fn parse(slice: &'a [u8], header_buf: &'headers mut [Header<'a>]) -> Result<Response<'a, 'headers>> {
    parse_http_version(slice)?;
    let mut offset: usize = 9;
    let (status, reason, read) = parse_status(&slice[offset..])?;
    offset += read;
    let read = parse_headers(&slice[offset..], header_buf)?;
    offset += read;
    Ok(Response::new(status, reason, header_buf, &slice[offset..]))
  }
}
impl<'a, 'headers> fmt::Display for Response<'a, 'headers> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Status line
        if self.reason.is_empty() {
            write!(f, "HTTP/1.1 {}\r\n", self.status)?;
        } else {
            write!(f, "HTTP/1.1 {} {}\r\n", self.status, self.reason)?;
        }

        // Headers
        for header in self.headers.iter() {
            if header.name.is_empty() {
                continue;
            }
            writeln!(f, "{}", header)?;
        }

        // Empty line after headers
        writeln!(f)?;

        // Body
        match core::str::from_utf8(self.body) {
            Ok(v) => write!(f, "{}", v),
            Err(_) => write!(f, "{:?}", self.body),
        }
    }
}


#[inline(always)]
fn parse_http_version(slice: &[u8]) -> Result<HttpVer> {
  match slice.get(0..9) {
    Some(b"HTTP/1.1 ") => Ok(HttpVer::One),
    Some(b"HTTP/2.0 ") => Ok(HttpVer::Two),
    _ => Err(Error::Malformed)
  }
}

#[inline(always)]
//parses the method and removes white space after it
//Returns the status, reason phrase, and bytes read
fn parse_status(slice: &[u8]) -> Result<(u16, &str, usize)> {
  for (counter, character) in slice.iter().enumerate() {
    // a number character
    if (48..=57).contains(character) {
      continue;
    } else if *character == SPACE {
      let status = &slice[..counter];
      if status.len() > 3 {
        return Err(Error::Malformed);
      }
      //there is a reason phrase
      if (65..=90).contains(&slice[counter+1]) || (97..=122).contains(&slice[counter+1]) {
        let reason = parse_reason(&slice[(counter+1)..])?;
        //SAFETY: already checked that the input is valid ascii
        return Ok((str::parse::<u16>(unsafe {core::str::from_utf8_unchecked(status)}).unwrap(), reason.0, counter + 1 + reason.1));
        //there is no reason phrase
      } else if slice[counter+1] == CR {
        if slice[counter+2] != LF {
          return Err(Error::Malformed);
        }
        //SAFETY: already checked that the input is valid ascii
        return Ok((str::parse::<u16>(unsafe {core::str::from_utf8_unchecked(status)}).unwrap(), "", counter + 3));
      } else {return Err(Error::Malformed);}
    } else if *character == CR {
      let status = &slice[..counter];
      if status.len() > 3 {
        return Err(Error::Malformed);
      }
      if slice[counter+1] != LF {
        return Err(Error::Malformed);
      }
      //SAFETY: already checked that the input is valid ascii
      return Ok((str::parse::<u16>(unsafe {core::str::from_utf8_unchecked(status)}).unwrap(), "", counter + 2));
    } else {
      return Err(Error::Malformed);
    }
  }
  Err(Error::Malformed)
}


#[inline(always)]
fn parse_reason(slice: &[u8]) -> Result<(&str, usize)> {
  for (counter, character) in slice.iter().enumerate() {
    if HEADER_NAME_SAFE[*character as usize] {
      continue;
    } else if *character == CR {
      let reason = &slice[..counter];
      if slice[counter+1] != LF {
        return Err(Error::Malformed);
      }
      //SAFETY: already checked that the input is valid ascii
      return Ok( (unsafe { core::str::from_utf8_unchecked(reason) }, counter+2));
    } else {
      return Err(Error::Malformed);
    }
  }
  Err(Error::Malformed)
}

