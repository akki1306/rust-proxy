use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use crate::cli::recv_input_user;
use crate::handler::handle_request;
use crate::site_blocking::ProxyData;

mod cli;
mod site_blocking;
mod parser;
mod stream_io;
mod handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let proxy_data = ProxyData {
        blocked_sites: Arc::new(Mutex::new(HashMap::new())),
        file_name: "blocked_sites.txt".to_string(),
    };

    loop {
        let (mut client_stream, _) = listener.accept().await?;
        proxy_data.load_blocked_sites().await?;
        let clone = proxy_data.clone();
        tokio::spawn(async move {
            let result = handle_request(&mut client_stream, clone).await;
            if let Err(e) = result {
                println!("Error occurred while handling request {:?}", e);
            }
        });
    }
}
