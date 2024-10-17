# Htpp

An **fast** and simple HTTP 1.1 parser written in Rust

# Contribution

Feel free to make a pull request if you think you can improve the code

# Usage

To parse an HTTP request:

```rust
let req = b"GET /index.html HTTP/1.1\r\n";
let parsed_req = htpp::RequestParser::new(req).parse().unwrap();
assert(parsed_req.method() == htpp::Method::Get);
assert(parsed_req.path() == "/index.html");
```

To parse an HTTP response:

```rust
let resp = b"HTTP/1.1 200 OK\r\n\r\n";
let parsed_resp = htpp::ResponseParser::new(resp).parse().unwrap();
assert(parsed_resp.status() == 200);
assert(parsed_resp.reason() == "OK");
```

