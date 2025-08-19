
#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

use core::fmt;

use crate::URL_SAFE;

#[cfg(feature = "no_std")]
use alloc::format;



/// An ampty query parameter for ease of parsing
pub const EMPTY_QUERY: QueryParam = QueryParam {name: "", val: ""};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// A URL query parameter
pub struct QueryParam<'a> {
    /// The name of the query parameter
    pub name: &'a str,
    /// The value of the query parameter
    pub val: &'a str,
}
impl<'a> fmt::Display for QueryParam<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.val)
    }
}
impl<'a> QueryParam<'a> {
  /// Create a new HTTP header with the given name and value
  pub fn new(name: &'a str, val: &'a str) -> Self {
    Self {
        name,
        val
    }
  }
}







/// All errors that could result from parsing a URL
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UrlError {
  /// An error while parsing the path part of the URL
  Path,
  /// An error while parsing the query parameters part of the URL
  Query,
  /// The URL has more query parameters than the length of the buffer passed
  TooManyQueryParams
}


#[derive(Debug, PartialEq, Eq, Clone)]
/// The path and query parameters part of an HTTP URL
pub struct Url<'a, 'queries> {
    /// The path part for the URL
    pub path: &'a str,
    /// The query parameters part for the URL
    pub query_params: Option<&'queries [QueryParam<'a>]>,
}

impl<'a, 'queries> Url<'a, 'queries> {
  /// Construct a new `Url` from its parts.
  pub fn new(path: &'a str, query_params: Option<&'queries [QueryParam<'a>]>) -> Url<'a, 'queries> {
    if path.is_empty() {
      return Self {
        path: "/",
        query_params,
      }
    }
    Self {
      path,
      query_params,
    }
  }


  /// Parses the bytes of an HTTP URL into a `Url`
  /// The URL you parse must be valid UTF-8 and must be stripped of the leading protocol and authority parts or an Err(UrlError::Path) is returned
  /// If you pass an empty `queries_buf`, it will not parse query parameters
  /// If there is more query parameters than the length of the passed `queries_buf`, an Err(UrlError::TooManyQueryParams) is returned
  #[inline]
  pub fn parse(slice: &'a [u8], queries_buf: &'queries mut [QueryParam<'a>]) -> Result<Url<'a, 'queries>, UrlError> {
    let mut offset = 0;
    let path = parse_path(slice)?;
    offset += path.1;
    if offset == slice.len() || queries_buf.is_empty(){
      return Ok(Url{path: path.0, query_params: None});
    }
    parse_query_params(&slice[offset..], queries_buf)?;
    Ok(Url{path: path.0, query_params: Some(queries_buf)})
  }
}


impl<'a, 'queries> fmt::Display for Url<'a, 'queries> {
    /// The string representation of the URL
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.query_params {
            Some(queries) => {
                // Write the base path
                write!(f, "{}", self.path)?;

                let mut first = true;
                for query in queries.iter() {
                    if query.name.is_empty() {
                        continue;
                    }
                    if first {
                        write!(f, "?{}={}", query.name, query.val)?;
                        first = false;
                    } else {
                        write!(f, "&{}={}", query.name, query.val)?;
                    }
                }
                Ok(())
            }
            None => write!(f, "{}", self.path),
        }
    }
}




#[inline(always)]
fn parse_path(slice: &[u8]) -> Result<(&str, usize), UrlError> {
  if slice.is_empty() || slice[0] != b'/' {return Err(UrlError::Path);}

  for (counter, character) in slice.iter().enumerate() {
    if *character == b'?' {
      let path = &slice[..counter];
      //SAFETY: already checked characters are valid UTF-8
      return Ok( (unsafe { core::str::from_utf8_unchecked(path) }, counter+1));
    }
  }
  //SAFETY: already checked characters are valid UTF-8
  Ok((unsafe { core::str::from_utf8_unchecked(slice) }, slice.len()))
}


#[inline(always)]
fn parse_query_params<'a>(slice: &'a [u8], queries_buf: &mut [QueryParam<'a>]) -> Result<(), UrlError> {
  let mut offset = 0;
  let mut iteration = 0;
  while offset < slice.len() {
    if iteration >= queries_buf.len() {return Err(UrlError::TooManyQueryParams);}
    let name = parse_query_param_name(&slice[offset..])?;
    offset += name.1;
    let val = parse_query_param_value(&slice[offset..])?;
    offset += val.1;
    queries_buf[iteration] = QueryParam::new(name.0, val.0);
    iteration += 1;
  };
  Ok(())
}


#[inline(always)]
// parses the header name and removes the `:` character and any spaces after it
fn parse_query_param_name(slice: &[u8]) -> Result<(&str, usize), UrlError> {
  for (counter, character) in slice.iter().enumerate() {
    if crate::HEADER_NAME_SAFE[*character as usize] {
      continue;
    } else if *character == b'=' {
      let query_name = &slice[..counter];
      if query_name.is_empty() {return Err(UrlError::Query);}
      //SAFETY: already checked characters are valid UTF-8
      return Ok( (unsafe { core::str::from_utf8_unchecked(query_name) }, counter+1));
    }
  }
  Err(UrlError::Query)
}

#[inline(always)]
fn parse_query_param_value(slice: &[u8]) -> Result<(&str, usize), UrlError> {
  for (counter, character) in slice.iter().enumerate() {
    if *character == b'&' {
      let val = &slice[..counter];
      if val.is_empty() {return Err(UrlError::Query);}
      //SAFETY: already checked characters are valid UTF-8
      return Ok( (unsafe { core::str::from_utf8_unchecked(val) }, counter+1));
    }
  }
  //SAFETY: already checked characters are valid UTF-8
  Ok((unsafe { core::str::from_utf8_unchecked(slice) }, slice.len()))
}













