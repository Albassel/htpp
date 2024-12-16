
use crate::{request::{self, Method}, response, Error, HttpVer};



macro_rules! req {
  ($name:ident, $buf:expr, |$arg:ident| $body:expr) => (
    #[test]
    fn $name() {
      let buf = $buf;
      let mut req = crate::request::Request::parse(buf).unwrap();
      closure(req);
      fn closure($arg: crate::request::Request) {
          $body
      }
      }
  );
  // a malformed request that shoud panic
  ($name:ident, $buf:expr, should_panic) => (
    #[test]
    #[should_panic]
    fn $name() {
      let buf = $buf;
      let mut req = crate::request::Request::parse(buf).unwrap();
      }
  );
}




req! {
    test_request_simple,
    b"GET / HTTP/1.1\r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 0);
    }
}

req! {
    test_request_simple_with_query_params,
    b"GET /thing?data=a HTTP/1.1\r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/thing?data=a");
        assert_eq!(req.headers.len(), 0);
    }
}

req! {
    test_request_headers,
    b"GET / HTTP/1.1\r\nHost: foo.com\r\nCookie: \r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0].name, "Host");
        assert_eq!(req.headers[0].val, b"foo.com");
        assert_eq!(req.headers[1].name, "Cookie");
        assert_eq!(req.headers[1].val, b"");
    }
}

req! {
    // test the scalar parsing
    test_request_header_value_htab_short,
    b"GET / HTTP/1.1\r\nUser-Agent: some\tagent\r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, "User-Agent");
        assert_eq!(req.headers[0].val, b"some\tagent");
    }
}

req! {
    // test the avx2 parsing
    test_request_header_value_htab_long,
    b"GET / HTTP/1.1\r\nUser-Agent: 1234567890some\t1234567890agent1234567890\r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, "User-Agent");
        assert_eq!(req.headers[0].val, &b"1234567890some\t1234567890agent1234567890"[..]);
    }
}

req! {
    test_request_header_no_space_after_colon,
    b"GET / HTTP/1.1\r\nUser-Agent:omg-no-space1234567890some1234567890agent1234567890\r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, "User-Agent");
        assert_eq!(req.headers[0].val, &b"omg-no-space1234567890some1234567890agent1234567890"[..]);
    }
}

req! {
    test_request_with_string_body,
    b"GET / HTTP/1.1\r\nUser-Agent: foo.com\r\n\r\na string body",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, "User-Agent");
        assert_eq!(req.headers[0].val, b"foo.com");
        assert_eq!(req.body, b"a string body");
    }
}

req! {
    test_request_with_non_utf8_body,
    b"GET / HTTP/1.1\r\nUser-Agent: foo.com\r\n\r\n\xe0\x3e\x38\x2e\x7e",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 1);
        assert_eq!(req.headers[0].name, "User-Agent");
        assert_eq!(req.headers[0].val, b"foo.com");
        assert_eq!(req.body, b"\xe0\x3e\x38\x2e\x7e");
    }
}

req! {
    test_request_multibyte,
    b"GET / HTTP/1.1\r\nHost: foo.com\r\nUser-Agent: \xe3\x81\xb2\xe3/1.0\r\n\r\n",
    |req| {
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/");
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0].name, "Host");
        assert_eq!(req.headers[0].val, b"foo.com");
        assert_eq!(req.headers[1].name, "User-Agent");
        assert_eq!(req.headers[1].val, b"\xe3\x81\xb2\xe3/1.0");
    }
}

req! {
    test_request_newlines,
    b"GET / HTTP/1.1\nHost: foo.bar\n\n",
    should_panic
}

req! {
    test_request_empty_lines_prefix,
    b"\r\n\r\nGET / HTTP/1.1\r\n\r\n",
    should_panic
}

req! {
    test_request_path_with_invalid_chars,
    b"GET /\\?wayne\\=5 HTTP/1.1\r\n",
    should_panic
}

req! {
    test_request_with_invalid_token_delimiter,
    b"GET\n/ HTTP/1.1\r\nHost: foo.bar\r\n\r\n",
    should_panic
}

req! {
    test_request_with_invalid_short_version,
    b"GET / HTTP/1!",
    should_panic
}

req! {
    test_request_with_empty_method,
    b" / HTTP/1.1\r\n\r\n",
    should_panic
}

req! {
    test_request_with_empty_path,
    b"GET  HTTP/1.1\r\n\r\n",
    should_panic
}






// --------------------------
//  TESTING RESPONSES
// --------------------------






macro_rules! res {
  ($name:ident, $buf:expr, |$arg:ident| $body:expr) => (
    #[test]
    fn $name() {
      let buf = $buf;
      let mut res = crate::response::Response::parse(buf).unwrap();
      closure(res);
      fn closure($arg: crate::response::Response) {
          $body
      }
      }
  );
  // a malformed request that shoud panic
  ($name:ident, $buf:expr, should_panic) => (
    #[test]
    #[should_panic]
    fn $name() {
      let buf = $buf;
      let mut res = crate::response::Response::parse(buf).unwrap();
      }
  );
}

res! {
    test_response_simple,
    b"HTTP/1.1 200 OK\r\n\r\n",
    |res| {
        assert_eq!(res.status, 200);
        assert_eq!(res.reason, "OK");
    }
}

 res! {
    test_response_newlines,
    b"HTTP/1.0 403 Forbidden\nServer: foo.bar\n\n",
    should_panic
}

 res! {
    test_response_reason_missing,
    b"HTTP/1.1 200 \r\n\r\n",
    |res| {
        assert_eq!(res.status, 200);
        assert_eq!(res.reason, "");
    }
}

res! {
    test_response_reason_missing_no_space,
    b"HTTP/1.1 200\r\n\r\n",
    |res| {
        
        assert_eq!(res.status, 200);
        assert_eq!(res.reason, "");
    }
}

res! {
    test_response_reason_missing_no_space_with_headers,
    b"HTTP/1.1 200\r\nFoo: bar\r\n\r\n",
    |res| {
        assert_eq!(res.status, 200);
        assert_eq!(res.reason, "");
        assert_eq!(res.headers.len(), 1);
        assert_eq!(res.headers[0].name, "Foo");
        assert_eq!(res.headers[0].val, b"bar");
    }
}

res! {
    test_response_reason_with_space_and_tab,
    b"HTTP/1.1 101 Switching Protocols\t\r\n\r\n",
    should_panic
}

res! {
    test_response_reason_with_obsolete_reason_byte,
    b"HTTP/1.1 200 X\xFFZ\r\n\r\n",
    should_panic
}

res! {
    test_response_reason_with_nul_byte,
    b"HTTP/1.1 200 \x00\r\n\r\n",
    should_panic
}

res! {
    test_response_version_missing_space,
    b"HTTP/1.1",
    should_panic
}

res! {
    test_response_code_missing_space,
    b"HTTP/1.1 200",
    should_panic
}

res! {
    test_response_partial_parses_headers_as_much_as_it_can,
    b"HTTP/1.1 200 OK\r\nServer: yolo\r\n",
    should_panic
}

res! {
    test_response_no_cr,
    b"HTTP/1.0 200\nContent-type: text/html\n\n",
    should_panic
}




// --------------------------
//  TESTING URL PARSING
// --------------------------




macro_rules! url {
  ($name:ident, $buf:expr, |$arg:ident| $body:expr) => (
    #[test]
    fn $name() {
      let buf = $buf;
      let mut queries = [crate::uri::EMPTY_QUERY; 10];
      let mut url = crate::uri::Url::parse(buf, &mut queries).unwrap();
      closure(url);
      fn closure($arg: crate::uri::Url) {
          $body
      }
      }
  );
}


url! {
    test_url,
    b"/path/path.html/user?query1=value&query2=value&query3=value",
    |url| {
        assert_eq!(url.path, "/path/path.html/user");
        assert_eq!(url.query_params.unwrap().len(), 3);
        assert_eq!(url.query_params.as_ref().unwrap()[0].name, "query1");
        assert_eq!(url.query_params.as_ref().unwrap()[0].val, "value");
    }
}


