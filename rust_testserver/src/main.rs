use std::{
    collections::HashMap, fs::{File}, io::{BufReader, prelude::*}, net::{TcpListener, TcpStream}, sync::{Arc, RwLock}, usize, vec
};

use rust_testserver::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6157").unwrap();
    let pool = ThreadPool::new(2);

    let cache: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));

    //listen for requests and foward them to the threadpool
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let map_clone= Arc::clone(&cache);

        pool.execute(|| {
            handle_connection(stream, map_clone);
        });
    }
}

fn handle_connection(stream: TcpStream, cache: Arc<RwLock<HashMap<String, String>>>) {
    let mut buf_reader = BufReader::new(&stream);

    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).unwrap();

    //read the request-line and set status-line/domain
    //filter for bad requests
    let (status_line, domain) = match request_line.contains("GET /dns-query") && request_line.contains("type=A") {
        true => {
            ("HTTP/1.1 200 OK", request_line.split("name=").nth(1).unwrap().split("&type=").nth(0).unwrap())
        }
        false => {
            ("HTTP/1.1 400 BAD REQUEST", "")
        }
    };

    //respond to bad request and do no further work
    if domain.is_empty() {
        respond_to_client(stream, status_line, String::new(), 0);
        return;
    }

    //find correct ip
    //search inside the cache first and the file after if needed
    let cache_reader = cache.read().unwrap();
    let ip = match cache_reader.get(domain) {
        Some(a) => a.to_string(),
        None => {
            let mut ip_adress = String::new();
            drop(cache_reader);

            let file = File::open("generated_hosts.txt").expect("Couldnt find/read recordfile");
            let mut records = BufReader::new(file);

            let prefix_string = format!("{} ", domain);
            let prefix = prefix_string.as_bytes();
            let mut line = Vec::new();
            while records.read_until(b'\n', &mut line).unwrap() != 0 {
                if line.starts_with(&prefix) {
                    line.strip_prefix(prefix).unwrap().strip_suffix(b"\r\n").unwrap().read_to_string(&mut ip_adress).unwrap();
                    let mut cache_writer = cache.write().unwrap();
                    cache_writer.insert(domain.to_string(), ip_adress.clone());
                    break;
                }
                line.clear();
            }

            if ip_adress.is_empty() {
                ip_adress.push_str("NODATA");
            }
            ip_adress
        }
    };

    let mut content_length = 0;

    //read the rest of the http headers
    //if additional headers must be searched/found, add them in the loop
    let mut line_buffer = String::new();
    loop {
        buf_reader.read_line(&mut line_buffer).unwrap();

        if line_buffer == "\r\n" {
            break;
        }

        let mut header = line_buffer.split_ascii_whitespace();

        if header.next().unwrap().eq_ignore_ascii_case("Content-Length:") {
            content_length = header.next().unwrap().parse::<usize>().unwrap();
        }

        line_buffer.clear();
    }

    //calculate the rest of the data needed for a correct response
    let mut body = vec![0; content_length];
    buf_reader.read_exact(&mut body).unwrap();
    let request_number = String::from_utf8(body).unwrap().trim().to_string();

    let body = format!("{ip},{request_number}");
    let body_length = body.len();

    respond_to_client(stream, status_line, body, body_length);
    return;
}

fn respond_to_client(mut stream: TcpStream, status_line: &str, body: String, body_size: usize) {
    let response = format!("{status_line}\r\nContent-Length: {body_size}\r\n\r\n{body}");
    stream.write_all(response.as_bytes()).unwrap();
}