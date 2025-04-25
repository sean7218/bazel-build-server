use std::{fs::OpenOptions, io::{self, BufRead, BufReader, Read, Write }};
mod json_rpc;

fn main() -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/users/sean7218/bazel/buildserver/output.txt")?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        let mut content_length = None;
        let mut buffer = String::new();
        loop {
            buffer.clear();
            let bytes = reader.read_line(&mut buffer)?;
            if bytes == 0 {
                return Ok(()); // EOF
            }

            if buffer == "\r\n" {
                break; // End of headers
            }

            if let Some(colon_position) = buffer.find(":") {
                let (key, value) = buffer.split_at(colon_position);
                if key.eq_ignore_ascii_case("Content-Length") {
                    content_length = value[1..].trim().parse::<usize>().ok();
                }
            }
        }

        let content_length = match content_length {
            Some(len) => len,
            None => {
                write!(file, "buildserver | error | Missing Content-Length header")?;
                continue;
            }
        };

        let mut body: Vec<u8> = vec![0; content_length];
        reader.read_exact(&mut body)?;

        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(json) => json,
            Err(e) => {
                writeln!(file, "buildserver | error | {:?}", e)?;
                continue;
            }
        };

        writeln!(file, "buildserver | request received | {:?}", request.to_string())?;

      
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "displayName": "bazel-build-erver",
                "version": "1.0.0",
                "bspVersion": "2.0",
                "capabilities": {
                    "compileProvider": true,            
                }
            }
        });
            
        let response_str = response.to_string();
        let response_len = response_str.len();

        write!(stdout, "Content-Length: {}\r\n", response_len)?;
        write!(stdout, "\r\n")?;
        write!(stdout, "{}", response_str)?;
        stdout.flush()?;

        writeln!(file, "buildserver | response send | {}", response_str)?;
    }
}
