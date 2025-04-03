use std::{fs::File, io::{Error, ErrorKind, Read, Write}, net::TcpStream};

struct Range {
    begin_index: u32,
    end_index: u32
}

struct HTTPResult {
    length: u32,
    body_bytes: Vec<u8>
}

fn build_get_request_string(range: Option<Range>) -> String {
    /*
     * Creates GET request string based on an optional range.
     */

    let mut return_string = String::from(
        "GET / HTTP/1.1\r\n\
        Accept: */*\r\n");

    if let Some(some_range) = range {
        return_string.push_str(format!("Range: bytes={}-{}\r\n", some_range.begin_index, some_range.end_index).as_str());
    }

    return_string.push_str("\r\n");

    return return_string;
}

fn recv_http(stream: &mut TcpStream) -> Result<HTTPResult, Box<dyn std::error::Error>> {
    /*
     * Receives HTTP data from TCP Stream.
     * If everything goes well, it should return the payload length sent in the headers, as well as the data.
     */

    let mut header = String::new();
    let mut body_bytes: Vec<u8> = Vec::new();
    let mut buffer: [u8; 500] = [0; 500];

    let mut in_header = true;
    let mut body_length: u32 = 0;
    let mut body_read_so_far: u32 = 0;

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        let mut i = 0;

        if bytes_read == 0 || (!in_header && body_length == body_read_so_far) {
            break;
        }

        while i < bytes_read {
            if i < bytes_read - 3 && &buffer[i..i+4] == b"\r\n\r\n" {
                // Header complete, get Content-Length and continue with reading the payload.
                in_header = false;
                for header_row in header.split("\r\n") {
                    let split_row: Vec<&str> = header_row.split(": ").collect();
                    if split_row.len() == 2 && split_row.iter().nth(0).unwrap() == &"Content-Length" {
                        body_length = split_row.iter().nth(1).unwrap().parse::<u32>()?;
                    }
                }

                i += 4;
            } else if in_header {
                header.push(buffer[i] as char);
                i += 1;
            } else {
                body_bytes.push(buffer[i]);
                i += 1;
                body_read_so_far += 1;
                if body_length == body_read_so_far {
                    break;
                }
            }
        }
    }

    if !in_header {
        return Ok(HTTPResult {length: body_length, body_bytes});
    }

    return Err(Box::new(Error::new(ErrorKind::Other, format!("Finished reading, but still in header: \n{}", header))));
}

fn send_all_tcp(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    /*
     * Send bytes on TCP stream.
     */

    let mut pos = 0;
    while pos < data.len() {
        let bytes_written = stream.write(&data[pos..])?;
        pos += bytes_written;
    }

    return Ok(());
}

fn main() -> std::io::Result<()> {
    let mut to_receive: u32 = 0;
    let mut known_size = false;
    let mut body: Vec<u8> = Vec::new();

    while !known_size || to_receive > 0 {
        // We ask the server for data multiple times until we have all of it.

        match TcpStream::connect("127.0.0.1:8080") {
            Ok(mut stream) => {
                let request = build_get_request_string(
                    match known_size {
                        false => None,
                        true => Some(Range {begin_index: body.len() as u32, end_index: (body.len() as u32 + to_receive)})
                    }
                );

                match send_all_tcp(&mut stream, request.as_bytes()) {
                    Ok(()) => {
                        println!("Sent request!");

                        match recv_http(&mut stream) {
                            Ok(http_result) => {
                                println!("Received {} bytes!", http_result.body_bytes.len());
                                if !known_size {
                                    known_size = true;
                                    to_receive = http_result.length;
                                }
                                body.extend(http_result.body_bytes.iter());
                                to_receive -= http_result.body_bytes.len() as u32;
                            },

                            Err(e) => {
                                println!("Error receiving data from server.");
                                println!("{}", e.to_string());
                                return Ok(());
                            }
                        }
                    },

                    Err(_) => {
                        println!("Failed to send request to server!");
                        return Ok(());
                    }
                }
            },

            Err(_) => {
                println!("Could not connect to server!");
                return Ok(());
            }
        }
    }

    println!("Done!");
    println!("Downloaded {} bytes", body.len());
    let mut output_file = File::create("downloaded.bin")?;
    output_file.write_all(body.as_slice())?;
    println!("Saved downloaded file as downloaded.bin");

    return Ok(());
}
