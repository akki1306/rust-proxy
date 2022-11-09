use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::site_blocking::ProxyData;


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let proxy_data = ProxyData {
        blocked_sites: Arc::new(Mutex::new(HashMap::new())),
        file_name: "blocked_sites.txt".to_string(),
    };
    proxy_data.load_blocked_sites().await;
    let handle = tokio::spawn(async move { recv_input_user(&proxy_data).await });
    tokio::join!(handle);
    Ok(())
}

pub async fn recv_input_user(proxy_data: &ProxyData) {
    loop {
        println!("\n*******************Please select one of the following*******************\n");
        println!("a. Block site\nb. List Blocked Sites\nc. Close");
        let mut s = String::new();
        print!("Your answer : ");
        stdout().flush();
        stdin().read_line(&mut s).expect("Please select one of the options above.");
        match s.trim().as_ref() {
            "a" => {
                print!("Please enter site to be blocked: ");
                stdout().flush();
                s.clear();
                stdin().read_line(&mut s).expect("please enter site url");
                let res = proxy_data.add_blocked_site(&s).await;
                match res {
                    Ok(true) => println!("*** site added successfully ***"),
                    Ok(false) => println!("!! site already exists !!"),
                    Err(e) => println!("Error occurred while adding site {:?}", e),
                }
            }
            "b" => {
                println!("\n**BLOCKED SITES LIST**\n");
                for key in proxy_data.blocked_sites.lock().await.iter() {
                    println!("{}", key.0);
                }
                println!("**********************");
            }
            _ => break,
        }
    }
}
