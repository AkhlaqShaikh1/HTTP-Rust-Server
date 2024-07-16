// Uncomment this block to pass the first stage
use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    str, thread,
};

const CRLF: &str = "\r\n";

fn handle_connection(mut stream: TcpStream) {
    println!("Connection established!");
    let mut buffer = [0; 2048];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]); // Log the entire request for debugging
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
            } else {
                let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
            }

            stream.flush().unwrap();
        }
        "POST" => {
            if tokens[1] == "/" {
                let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\n");
            } else if tokens[1].starts_with("/files/") {
                let file_name = tokens[1].replace("/files/", "");
                let mut contents = String::new();
                let content_length = lines[4].replace("Content-Length: ", "").parse::<usize>();
                let cl = content_length.unwrap();
                for i in 0..cl {
                    contents.push(lines[7].chars().nth(i).unwrap());
                }
                if let Some(dir) = env::args().nth(2) {
                    let _ = fs::write(Path::new(&dir).join(file_name), contents);
                    let _ = stream.write(b"HTTP/1.1 201 OK\r\n\r\n");
                } else {
                    let _ = stream.write(b"HTTP/1.1 500 Internal Server Error\r\n\r\n");
                }
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
