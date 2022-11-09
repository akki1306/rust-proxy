#[derive(Default)]
pub struct HttpRequestAttributes {
    pub url: Option<String>,
    pub host_name: Option<String>,
    pub port_number: Option<String>,
    pub ip_address: Option<String>,
}

impl HttpRequestAttributes {
    pub fn parse(&mut self, request_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.fetch_target_server_details(request_data)?;
        Ok(())
    }

    fn construct_url(&mut self, request_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut result = String::new();
        let mut index_first_space_char = request_data.find(' ').ok_or("error parsing url string")?;
        let mut url_string: &str = &request_data[index_first_space_char + 1..]; // discard request type from request data
        index_first_space_char = url_string.find(' ').ok_or("error parsing url string")?;
        url_string = &url_string[0..index_first_space_char]; // extract url string

        if &url_string[0..4] != "http" {
            result.push_str(&*format!("http://{}", url_string));
        } else {
            result.push_str(url_string);
        }

        self.url = Some(result);
        Ok(())
    }

    fn fetch_target_server_details(
        &mut self, request_data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.construct_url(request_data).unwrap();
        let url_string = &self.url.as_ref().cloned().unwrap()[7..];
        let split = url_string.split(":").collect::<Vec<&str>>();
        let host_name = split.get(0).ok_or("error parsing host name")?;
        let port_number = split.get(1).ok_or("error parsing port number")?;
        let target_server_ip_addr = dns_lookup::lookup_host(host_name);
        for ip in target_server_ip_addr.unwrap().into_iter() {
            if ip.is_ipv4() {
                self.host_name = Some(host_name.to_string());
                self.port_number = Some(port_number.to_string());
                self.ip_address = Some(ip.to_string());
                break;
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_prefix_http_to_url() {
        let mut http_request_attrs = HttpRequestAttributes::default();

        http_request_attrs.construct_url("GET www.google.com 123").unwrap();

        assert_eq!(http_request_attrs.url.unwrap(), "http://www.google.com")
    }

    #[test]
    fn should_prefix_nothing_if_protocol_present() {
        let mut http_request_attrs = HttpRequestAttributes::default();

        http_request_attrs.construct_url("GET http://www.google.com 123").unwrap();

        assert_eq!(http_request_attrs.url.unwrap(), "http://www.google.com");
    }

    #[test]
    fn should_prefix_nothing_for_https() {
        let mut http_request_attrs = HttpRequestAttributes::default();

        http_request_attrs.construct_url("GET https://www.google.com 123").unwrap();

        assert_eq!(http_request_attrs.url.unwrap(), "https://www.google.com");
    }

    #[test]
    fn should_throw_error_for_invalid_request_data() {
        let mut http_request_attrs = HttpRequestAttributes::default();

        if let Err(e) = http_request_attrs.construct_url("123") {
            assert_eq!("error parsing url string", e.to_string());
        }
    }

    #[test]
    fn should_fetch_target_server_details() {
        let mut http_request_attrs = HttpRequestAttributes::default();

        http_request_attrs.fetch_target_server_details("GET http://www.google.com:123 test");

        assert_eq!(http_request_attrs.url.unwrap(), "http://www.google.com:123");
        assert_eq!(http_request_attrs.host_name.as_ref().unwrap(), "www.google.com");
        assert_eq!(http_request_attrs.port_number.unwrap(), "123");

        for ip in dns_lookup::lookup_host(http_request_attrs.host_name.as_ref().unwrap()).unwrap().into_iter() {
            if ip.is_ipv4() {
                assert_eq!(http_request_attrs.ip_address.as_ref().unwrap(), &ip.to_string());
            }
        }

    }
}

