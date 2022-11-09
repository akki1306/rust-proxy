use std::ops::Deref;

use tokio::net::TcpStream;

use crate::parser::HttpRequestAttributes;
use crate::site_blocking::ProxyData;
use crate::stream_io::{read_from_stream, write_to_stream};

pub const ERROR_MSG_HTTP_403_FORBIDDEN: &str = concat!("HTTP/1.0 403 Access Forbidden \n",
"User-Agent: ProxyServer/1.0\n",
"\r\n");

pub const SUCCESS_MSG_HTTP_200_OK: &str = concat!(
"HTTP/1.0 200 Connection established\r\n",
"Proxy-Agent: ProxyServer/1.0\r\n",
"\r\n"
);

pub async fn handle_request(
    stream: &mut TcpStream,
    proxy_data: ProxyData,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.readable().await?;
    let result = read_from_stream(stream).await.unwrap();
    let mut http_request_attributes = HttpRequestAttributes::default();

    http_request_attributes.parse(result.clone().as_str())?;

    if proxy_data.is_blocked(&http_request_attributes.host_name.unwrap()).await? {
        write_to_stream(stream, ERROR_MSG_HTTP_403_FORBIDDEN).await?;
        return Ok(());
    }

    let request = &result[0..result.find(' ').ok_or("error parsing string")?];
    if request == "CONNECT" {
        let mut target_server_stream =
            TcpStream::connect(format!("{}:{}", &http_request_attributes.ip_address.unwrap(), &http_request_attributes.port_number.unwrap())).await?;
        read_from_stream(stream).await?; // to discard remaining data coming from client stream
        write_to_stream(stream, SUCCESS_MSG_HTTP_200_OK).await?;
        tokio::io::copy_bidirectional(stream, &mut target_server_stream).await?;
    } else {
        let response = connect_to_remote_server(&http_request_attributes.url.unwrap().as_str()).await?;
        write_to_stream(stream, response.deref()).await?;
    }
    Ok(())
}

async fn connect_to_remote_server(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let response_str = String::from_utf8(response.bytes().await?.to_vec())?;
    Ok(response_str)
}