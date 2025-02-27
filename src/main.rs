// Uncomment this block to pass the first stage
use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    str, thread,
};

use flate2::{read::GzEncoder, Compression};

const CRLF: &str = "\r\n";

fn handle_connection(stream: &mut TcpStream) {
    println!("Connection established!");
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    let lines: Vec<&str> = request.split("\r\n").collect();
    let tokens: Vec<&str> = lines[0].split(" ").collect();

    match tokens[0] {
        "GET" => {
            if tokens[1] == "/" {
                let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\n");
            } else if tokens[1].starts_with("/echo/") {
                handle_echo(stream, lines, tokens);
            } else if tokens[1].starts_with("/user-agent") {
                handle_user_agent(stream, lines);
            } else if tokens[1].starts_with("/files/") {
                handle_files(stream, tokens);
            } else {
                let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
            }

            stream.flush().unwrap();
        }
        "POST" => {
            if tokens[1] == "/" {
                let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\n");
            } else if tokens[1].starts_with("/files/") {
                handle_post(stream, tokens, lines);
            }
        }
        _ => {
            println!("Unknown method: {}", tokens[0])
        }
    }
}
fn handle_files(stream: &mut TcpStream, tokens: Vec<&str>) {
    if let Some(dir) = env::args().nth(2) {
        let file_name = tokens[1].replace("/files/", "");
        if let Ok(mut file) = fs::File::open(Path::new(&dir).join(file_name)) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let _ = stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents).as_bytes());
        } else {
            let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
        }
    }
}
fn handle_echo(stream: &mut TcpStream, lines: Vec<&str>, tokens: Vec<&str>) {
    let response = tokens[1].replace("/echo/", "");
    let mut encoding = String::new();
    for lines in lines.iter() {
        if lines.starts_with("Accept-Encoding: ") {
            encoding = lines.replace("Accept-Encoding: ", "");
            break;
        }
    }

    if encoding.contains("gzip") {
        handle_compression(stream, tokens);
    }
    let _ = stream.write(
        format!(
            "HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{}",
            response.len(),
            response
        )
        .as_bytes(),
    );
}

fn handle_compression(stream: &mut TcpStream, tokens: Vec<&str>) {
    let new_resp = tokens[1].replace("/echo/", "");
    let body = new_resp.as_bytes();
    let mut compbody: Vec<u8> = Vec::new();
    let mut encoder = GzEncoder::new(&body[..], Compression::default());
    encoder.read_to_end(&mut compbody).unwrap();
    let _ = stream.write(format!("HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Encoding: gzip{CRLF}Content-Length: {}{CRLF}{CRLF}",compbody.len() ).as_bytes(),);
    let _ = stream.write(&compbody);
    return;
}

fn handle_user_agent(stream: &mut TcpStream, lines: Vec<&str>) {
    let mut user_agent = String::new();
    for line in lines.iter() {
        if line.starts_with("User-Agent: ") {
            user_agent = line.replace("User-Agent: ", "");
            break;
        }
    }
    let _ = stream.write(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            user_agent.len(),
            user_agent
        )
        .as_bytes(),
    );
}

fn handle_post(stream: &mut TcpStream, tokens: Vec<&str>, lines: Vec<&str>) {
    let file_name = tokens[1].replace("/files/", "");
    let mut content_length = 0;
    let mut is_header = true;
    let mut body_start_index = 0;

    // Find Content-Length header and the start of the body
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("Content-Length: ") {
            content_length = line
                .replace("Content-Length: ", "")
                .parse::<usize>()
                .unwrap();
        }
        if line.is_empty() && is_header {
            body_start_index = index + 1;
            is_header = false;
        }
    }

    // Read the body
    let mut contents = String::new();
    for line in &lines[body_start_index..] {
        contents.push_str(line);
        contents.push_str("\r\n");
    }
    contents = contents.trim_end().to_string(); // Remove trailing \r\n

    if contents.len() > content_length {
        contents.truncate(content_length);
    }

    if let Some(dir) = env::args().nth(2) {
        let _ = fs::write(Path::new(&dir).join(file_name), contents);
        let _ = stream.write(b"HTTP/1.1 201 Created\r\n\r\n");
    } else {
        let _ = stream.write(b"HTTP/1.1 500 Internal Server Error\r\n\r\n");
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(move || handle_connection(&mut _stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
