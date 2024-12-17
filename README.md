# Htpp

A **fast** and simple HTTP 1.1 parser written in Rust

# Usage

You can parse a request as follows:

```rust
use htpp::{Request, EMPTY_HEADER};

let req = b"GET /index.html HTTP/1.1\r\n\r\n";
let mut headers = [EMPTY_HEADER; 10];
let parsed = Request::parse(req, &mut headers).unwrap();
assert!(parsed.method == htpp::Method::Get);
assert!(parsed.path == "/index.html");
```
You can create a request as follows:

```rust
use htpp::{Method, Request, Header};

let method = Method::Get;
let path = "/index.html";
let mut headers = [Header::new("Accept", b"*/*")];
let req = Request::new(method, path, &headers, b"");
```
## Working with [Response]

You can parse a response as follows:

```rust
use htpp::{Response, EMPTY_HEADER};

let req = b"HTTP/1.1 200 OK\r\n\r\n";
let mut headers = [EMPTY_HEADER; 10];
let parsed = Response::parse(req, &mut headers).unwrap();
assert!(parsed.status == 200);
assert!(parsed.reason == "OK");
```

You can create a response as follows:

```rust
use htpp::{Response, Header};

let status = 200;
let reason = "OK";
let mut headers = [Header::new("Connection", b"keep-alive")];
let req = Response::new(status, reason, &mut headers, b"");
```

After parsing a request, you can also parse the path part of the request inclusing query parameters as follows:

```rust
use htpp::{Request, EMPTY_QUERY, Url, EMPTY_HEADER};

let req = b"GET /index.html?query1=value&query2=value HTTP/1.1\r\n\r\n";
let mut headers = [EMPTY_HEADER; 10];
let parsed_req = Request::parse(req, &mut headers).unwrap();
let mut queries_buf = [EMPTY_QUERY; 10];
let url = Url::parse(parsed_req.path.as_bytes(), &mut queries_buf).unwrap();
assert!(url.path == "/index.html");
assert!(url.query_params.unwrap()[0].name == "query1");
assert!(url.query_params.unwrap()[0].val == "value");
```


# Contribution

Feel free to make a pull request if you think you can improve the code

