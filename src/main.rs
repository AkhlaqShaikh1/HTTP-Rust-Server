// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

const CRLF: &str = "\r\n";

fn handle_connection(mut stream: TcpStream) {
    println!("Connection established!");
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Request: {}", request); // Log the entire request for debugging
    let lines: Vec<&str> = request.split("\r\n").collect();
    let tokens: Vec<&str> = lines[0].split(" ").collect();
    match tokens[0] {
        "GET" => {
            if tokens[1] == "/" {
                let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\n");
            } else if tokens[1].starts_with("/echo/") {
                let response = tokens[1].replace("/echo/", "");
                let _ = stream.write(format!("HTTP/1.1 200 OK{CRLF}Content-Type: text/plain{CRLF}Content-Length: {}{CRLF}{CRLF}{}", response.len(), response).as_bytes());
            } else if tokens[1].starts_with("/user-agent") {
                let mut user_agent = String::new();
                for line in lines.iter() {
                    if line.starts_with("User-Agent: ") {
                        user_agent = line.replace("User-Agent: ", "");
                        break;
                    }
                }
                let _ = stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent).as_bytes());
            } else if tokens[1].starts_with("/files/") {
                let file_name = tokens[1].replace("/files/", "");
                println!("File name: {}", file_name);
                let file = std::fs::read_to_string(file_name);
                println!("File: {:?}", file);
                match file {
                    Ok(content) => {
                        let _ = stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", content.len(), content).as_bytes());
                    }
                    Err(_) => {
                        let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
                    }
                }
            } else {
                let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
            }
        }
        _ => {
            println!("Unknown method: {}", tokens[0])
        }
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
            Ok(_stream) => {
                thread::spawn(|| handle_connection(_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
