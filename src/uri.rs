

#![allow(unused)]
#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]

use std::{collections::HashMap, fmt, path::Path};

use crate::URL_SAFE;

/// All errors that could result from parsing a URL
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UrlError {
  /// An error while parsing the path part of the URL
  Path,
  /// An error while parsing the query parameters part of the URL
  Query,
}


#[derive(Debug, PartialEq, Eq, Clone)]
/// The path and query parameters part of an HTTP URL
pub struct Url<'a> {
    /// The path part for the URL
    pub path: &'a str,
    /// The query parameters part for the URL
    pub query_params: Option<HashMap<&'a str, &'a str>>,
}


impl<'a> Url<'a> {
  /// Construct a new `Url` from its parts.
  /// Use an empty `&str` to create a `Respose` with no reason phrase
  /// Use an empty `&str` to create a `Respose` with no body
  pub fn new(path: &'a str, query_params: Option<HashMap<&'a str, &'a str>>) -> Url<'a> {
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
  // The URL you parse must be valid UTF-8 and must be stripped of the leading protocol and authority parts or an Err is returned
  #[inline]
  pub fn parse(slice: &'a [u8]) -> Result<Url, UrlError> {
    let mut offset = 0;
    let path = parse_path(slice)?;
    offset += path.1;
    if offset == slice.len() {
      return Ok(Url{path: path.0, query_params: None});
    }
    let params = parse_query_params(&slice[offset..])?;
    Ok(Url{path: path.0, query_params: Some(params)})
  }
}


impl<'a> fmt::Display for Url<'a> {
    /// The string representation of the URL
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match &self.query_params {
        Some(map) => {
          let mut params = String::new();
          for (query, val) in map.iter() {
            params.push_str(query);
            params.push('=');
            params.push_str(val);
            params.push('&');
          }
          params.pop();
          f.write_str(format!("{}?{}", self.path, params).as_str())
        },
        None => f.write_str(self.path),
      }
    }
}






#[inline]
fn parse_path(slice: &[u8]) -> Result<(&str, usize), UrlError> {
  if slice.is_empty() || slice[0] != b'/' {return Err(UrlError::Path);}

  for (counter, character) in slice.iter().enumerate() {
    if *character == b'?' {
      let path = &slice[..counter];
      //SAFETY: already checked characters are valid UTF-8
      return Ok( (unsafe { std::str::from_utf8_unchecked(path) }, counter+1));
    }
  }
  //SAFETY: already checked characters are valid UTF-8
  Ok((unsafe { std::str::from_utf8_unchecked(slice) }, slice.len()))
}


#[inline]
fn parse_query_params(slice: &[u8]) -> Result<HashMap<&str, &str>, UrlError> {
  let mut query_params = HashMap::new();
  let mut offset = 0;
  while offset < slice.len() {
    let name = parse_query_param_name(&slice[offset..])?;
    offset += name.1;
    let val = parse_query_param_value(&slice[offset..])?;
    offset += val.1;
    query_params.insert(name.0, val.0);
  }
  Ok(query_params)
}


#[inline]
// parses the header name and removes the `:` character and any spaces after it
fn parse_query_param_name(slice: &[u8]) -> Result<(&str, usize), UrlError> {
  for (counter, character) in slice.iter().enumerate() {
    if crate::HEADER_NAME_SAFE[*character as usize] {
      continue;
    } else if *character == b'=' {
      let query_name = &slice[..counter];
      if query_name.is_empty() {return Err(UrlError::Query);}
      //SAFETY: already checked characters are valid UTF-8
      return Ok( (unsafe { std::str::from_utf8_unchecked(query_name) }, counter+1));
    }
  }
  unreachable!();
}

#[inline]
fn parse_query_param_value(slice: &[u8]) -> Result<(&str, usize), UrlError> {
  for (counter, character) in slice.iter().enumerate() {
    if *character == b'&' {
      let val = &slice[..counter];
      if val.is_empty() {return Err(UrlError::Query);}
      //SAFETY: already checked characters are valid UTF-8
      return Ok( (unsafe { std::str::from_utf8_unchecked(val) }, counter+1));
    }
  }
  //SAFETY: already checked characters are valid UTF-8
  Ok((unsafe { std::str::from_utf8_unchecked(slice) }, slice.len()))
}













