use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn write_to_stream(stream: &mut TcpStream, str: &str) -> Result<(), Box<dyn std::error::Error>> {
    stream.write_all(str.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn read_from_stream(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut tmp_buf = [0; 4096];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(&mut tmp_buf) {
            Ok(0) => {
                break;
            }
            Ok(_) => buf.extend_from_slice(&tmp_buf),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(std::str::from_utf8(&buf).unwrap().to_string())
}
