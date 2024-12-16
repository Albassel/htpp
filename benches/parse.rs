

use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const REQ_SHORT: &[u8] = b"GET / HTTP/1.1\r\n\
Host: example.com\r\n\
Cookie: session=60; user_id=1\r\n\r\n";

const REQ: &[u8] = b"GET /wp-content/uploads/pink.jpg HTTP/1.1\r\n\
Host: www.kittyhell.com\r\n\
User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
Accept-Encoding: gzip,deflate\r\n\
Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
Keep-Alive: 115\r\n\
Connection: keep-alive\r\n\
Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256\r\n\r\n";

fn req(c: &mut Criterion) {
  c.benchmark_group("req")
    .bench_function("req", |b| b.iter(|| {
      black_box(htpp::Request::parse(REQ).unwrap());
    }));
}


fn req_short(c: &mut Criterion) {
  c.benchmark_group("req_short")
    .bench_function("req_short", |b| b.iter(|| {
      black_box(htpp::Request::parse(REQ_SHORT).unwrap());
  }));
}



//--------------
// Benchmarking response parsing
//--------------



const RESP_SHORT: &[u8] = b"HTTP/1.1 200 OK\r\n\
Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
Set-Cookie: session=60; user_id=1\r\n\r\n";

// These particular headers don't all make semantic sense for a response, but they're syntactically valid
const RESP: &[u8] = b"HTTP/1.1 200 OK\r\n\
Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
Host: www.kittyhell.com\r\n\
User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
Accept-Encoding: gzip,deflate\r\n\
Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
Keep-Alive: 115\r\n\
Connection: keep-alive\r\n\
Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256\r\n\r\n";

fn resp(c: &mut Criterion) {
  c.benchmark_group("resp")
    .bench_function("resp", |b| b.iter(|| {
      black_box(htpp::Response::parse(RESP).unwrap());
  }));
}

fn resp_short(c: &mut Criterion) {
  c.benchmark_group("resp_short")
    .bench_function("resp_short", |b| b.iter(|| {
      black_box(htpp::Response::parse(RESP_SHORT).unwrap());
  }));
}




//--------------
// Benchmarking URL parsing
//--------------



const URL: &[u8] = b"/path/path.html/user?query1=value&query2=value&query3=value";

fn url(c: &mut Criterion) {
  c.benchmark_group("url")
  .bench_function("url", |b| b.iter_batched_ref(|| {
      [htpp::EMPTY_QUERY; 10]
  },|queries| {
    black_box(htpp::Url::parse(URL, queries).unwrap());
  }, criterion::BatchSize::SmallInput));
}




//--------------
// Running the benchmarks
//--------------



const WARMUP: Duration = Duration::from_millis(500);
const MTIME: Duration = Duration::from_millis(100);
const SAMPLES: usize = 400;
criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(SAMPLES).warm_up_time(WARMUP).measurement_time(MTIME);
    targets = req, req_short, resp, resp_short, url
}
criterion_main!(benches);