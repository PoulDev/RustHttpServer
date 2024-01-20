use std::io::{BufRead, Write, BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::time::SystemTime;

fn main() {
    let listener = TcpListener::bind("localhost:1337").unwrap();
    print!("Hosting on localhost:1337\n");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        print!("Connection {:?} -> ", stream.peer_addr().unwrap());

        let start = SystemTime::now();
        handle_connection(stream);
        print!(" ({}ms)\n", start.elapsed().unwrap().as_micros());
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    /*
     let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();*/ 
    let request_line = match buf_reader.lines().next() {
        Some(Ok(data)) => data,
        Some(Err(_)) => return,
        None => return,
    };

    /*
     * Old method to handle decoding errors
    let request_line = match request_data.unwrap() {
        Ok(data) => data,
        Err(e) => {
            print!("Decoding Failed: {:?}", e);
            return 
        },
    };*/
    
    let file_name = get_filename(request_line);
    let status: &str;
    let content: Vec<u8>;

    if file_name.contains("../") {
        print!("CWE-23 Blocked");
        status = "400 Bad Request";
        content = "Molto divertente.\n\r".as_bytes().to_vec();
    } else {
        print!("Providing File {}", file_name);

        content = match fs::read(file_name) {
            Ok(data) => {
                status = "200 OK";
                data
            },
            Err(_) => {
                status = "404 Not Found";
                fs::read("./src/html/404.html").unwrap()
            },
        };
    }

    send_response(stream, content, status.to_owned());
}

fn get_filename(request_line: String) -> String {
    let mut file_name: String = "".to_owned();

    // Trova il percorso in "GET /percorso HTTP/1.1"
    for part in request_line.split(" ") {
        if part.chars().nth(0) == Some('/') {
            file_name = "./src/html".to_owned() + part;
        }
    }

    // Aggiunge "index.html" se il percorso finisce con /
    // ( cioe' indica una cartella o accede ad / )
    if file_name.chars().nth(file_name.len()-1) == Some('/') {
        file_name += "index.html";
    }
    return file_name;
}

fn send_response(stream: TcpStream, content: Vec<u8>, status: String) {
    let mut stream_clone = stream.try_clone().unwrap();
    let mut buf_writer = BufWriter::new(&mut stream_clone);

    // Manda la risposta
    let len = content.len();
    let response = format!("HTTP/1.1 {status}\r\nContent-Length: {len}\r\nHost: Server di traba in rust uau\r\n\r\n");
    let response = [response.as_bytes(), &content].concat();

    buf_writer.write(&response).unwrap();
    buf_writer.flush().unwrap();
}
