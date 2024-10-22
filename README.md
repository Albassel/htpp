# Htpp

A **fast** and simple HTTP 1.1 parser written in Rust

# Usage

To parse an HTTP request:

```rust
let req = b"GET /index.html HTTP/1.1\r\nAccept: */*\r\n\r\n";
let parsed = htpp::Request::parse(req).unwrap();
assert!(parsed.method() == htpp::Method::Get);
assert!(parsed.path() == "/index.html");
assert!(parsed.headers().len() == 1);
assert!(parsed.headers()[0].name == "Accept");
assert!(parsed.headers()[0].val == b"*/*");
```

To parse an HTTP response:

```rust
let res = b"HTTP/1.1 200 OK\r\nAccept: */*\r\n\r\n";
let parsed = htpp::Response::parse(res).unwrap();
assert!(parsed.status() == 200);
assert!(parsed.reason() == "OK");
assert!(parsed.headers().len() == 1);
assert!(parsed.headers()[0].name == "Accept");
assert!(parsed.headers()[0].val == b"*/*");
```

# Contribution

Feel free to make a pull request if you think you can improve the code

