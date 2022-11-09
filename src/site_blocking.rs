use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct ProxyData {
    pub blocked_sites: Arc<Mutex<HashMap<String, String>>>,
    pub file_name: String,
}

impl ProxyData {
    pub async fn load_blocked_sites(&self) -> Result<(), Box<dyn Error>> {
        let file = File::open(&self.file_name).unwrap();
        let buf = BufReader::new(file);
        let lines: Vec<String> = buf.lines().map(|l| l.expect("could not parse line")).collect();
        for l in lines {
            self.blocked_sites.lock().await.insert(l.to_string(), l.to_string());
        }
        Ok(())
    }

    pub(crate) async fn is_blocked(&self, host_name: &String) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.blocked_sites.lock().await.contains_key(host_name))
    }

    pub async fn add_blocked_site(&self, site: &str) -> Result<bool, Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .append(true)
            .read(true)
            .open(&self.file_name)
            .unwrap();

        let buf = BufReader::new(&file);
        let lines = buf.lines();
        let sites = lines
            .map(|line| line.unwrap())
            .filter(|line| line == &site.trim().to_string())
            .count();
        if sites == 0 {
            file.write(site.as_bytes()).unwrap();
            file.flush().unwrap();
            self.blocked_sites.lock().await.insert(site.to_string(), site.to_string());
        }
        Ok(sites == 0)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;

    use tokio::sync::Mutex;
    use uuid::Uuid;
    use crate::ProxyData;

    #[tokio::test]
    async fn should_load_blocked_sites() {
        let (proxy_data, path) = create_proxy_data();
        proxy_data.load_blocked_sites().await.unwrap();

        assert_eq!(2, proxy_data.blocked_sites.lock().await.len());

        cleanup(path);
    }

    #[tokio::test]
    async fn should_check_site_blocked() {
        let (proxy_data, path) = create_proxy_data();

        proxy_data.blocked_sites.lock().await.insert("test".to_string(), "test".to_string());

        assert_eq!(true, proxy_data.is_blocked(&"test".to_string()).await.unwrap());
        assert_eq!(false, proxy_data.is_blocked(&"test2".to_string()).await.unwrap());

        cleanup(path);
    }

    #[tokio::test]
    async fn should_add_blocked_site() {
        let (proxy_data, path) = create_proxy_data();

        proxy_data.add_blocked_site("test3.com").await.unwrap();

        assert_eq!(true, proxy_data.blocked_sites.lock().await.contains_key("test3.com"));

        cleanup(path);
    }

    fn create_proxy_data() -> (ProxyData, String) {
        let uuid = Uuid::new_v4().to_string();
        let path = uuid + ".txt";
        let mut file = File::create(&path).unwrap();
        file.write("test\n".as_bytes()).unwrap();
        file.write("test1".as_bytes()).unwrap();
        file.flush().unwrap();

        (ProxyData { blocked_sites: Arc::new(Mutex::new(HashMap::new())), file_name: path.clone() }, path)
    }

    fn cleanup(path: String) {
        std::fs::remove_file(path).unwrap();
    }
}