use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

struct HTTPConnection {
    addr: String,
    content_length: u32,
}

impl HTTPConnection {
    fn new(addr: &str) -> HTTPConnection {
        let mut content_length = 0;

        let mut connection =
            TcpStream::connect(&addr).expect("Unsuccessful connection to the server");
        let _ = connection.set_read_timeout(Some(Duration::new(5, 0)));
        let _ = connection.set_write_timeout(Some(Duration::new(5, 0)));
        let request = format!(
            "GET / HTTP/1.1\r\n\
            Host: {}\r\n\
            Connection: close\r\n\
            \r\n",
            addr
        );
        connection
            .write_all(request.as_bytes())
            .expect("Unsuccessful request");
        let mut response: Vec<u8> = Vec::new();
        connection
            .read_to_end(&mut response)
            .expect("Unsuccessful receiving of the response");
        let mut response_string = String::new();
        for num in response {
            response_string.push(num as char);
        }

        for line in response_string.lines() {
            if line.starts_with("Content-Length:") {
                content_length = line
                    .split(" ")
                    .nth(1)
                    .expect("Unsuccessful parsing of the content length")
                    .trim()
                    .parse()
                    .expect("Unsuccssesful parsing of the content length");
            }
        }
        if content_length == 0 {
            panic!("Unable to find content length");
        }
        HTTPConnection {
            addr: addr.to_string(),
            content_length,
        }
    }
    fn download_segment(&self, destination: &mut [u8], start: usize) {
        let mut segment: Vec<u8> = Vec::new();
        segment.clear();
        let mut connection =
            TcpStream::connect(&self.addr).expect("Unsuccessful connection to the server");
        let _ = connection.set_read_timeout(Some(Duration::new(5, 0)));
        let _ = connection.set_write_timeout(Some(Duration::new(5, 0)));
        let request = format!(
            "GET / HTTP/1.1\r\n\
            Host: {}\r\n\
            Connection: close\r\n\
            Range: bytes={}-{}\r\n\
            \r\n",
            &self.addr,
            start,
            start + destination.len(),
        );
        println!("Segment {}-{}", start, start + destination.len());
        connection
            .write_all(request.as_bytes())
            .expect("Unsuccessful request");
        connection
            .read_to_end(&mut segment)
            .expect("Unsuccessful receiving of range response");
        let content_beginning = segment
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("Unable to find end of head in segment")
            + 4;
        destination.copy_from_slice(&segment[content_beginning..]);
    }
}

fn main() {
    let connection = HTTPConnection::new("127.0.0.1:8080");
    let mut data: Vec<u8> = vec![0; connection.content_length as usize];
    let mut left: &mut [u8];
    let mut right = &mut data[..];
    let mut start = 0;
    let packet_size = 64000;

    while right.len() > packet_size {
        (left, right) = right.split_at_mut(packet_size);
        connection.download_segment(left, start);
        start += packet_size;
    }
    if right.len() > 0 {
        connection.download_segment(right, start);
    }

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    println!("SHA-256 hash of the data: {:x}", result);
}
