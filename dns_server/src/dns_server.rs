use ipgeolocate::{GeoError, Locator, Service};
use geoutils::Location;
use std::{collections::HashMap};

pub struct DnsServer {
    port: String,
    // hashmap to store the CDN IP address and availability
    pub cdn_server: HashMap<String, bool>,
}

impl DnsServer {
    // This function is used to create a new instance of the DnsServer struct
    pub fn new() -> Self {
        DnsServer {
            port: String::from("20310"),
            cdn_server: HashMap::new(),
        }
    }
    // This function is used to get the geolocation of an IP address
    pub async fn get_geolocation(&self,ip: &str) -> Result<Locator, GeoError>{
        let service = Service::IpApi;
        let locator = match Locator::get(ip, service).await {
            Ok(locator) => locator,
            Err(error) => return Err(error),
        };
        Ok(locator)
    }
    
    // This function is used to get the distance between two IP addresses
    pub async fn get_distance_from_ip(&self, ip: &str, target_ip: &str) -> f64 {
        let locator = self.get_geolocation(ip).await.unwrap();
        let target_locator = self.get_geolocation(target_ip).await.unwrap();
        // Calculate the distance between the two IP addresses
        let locator = Location::new(locator.latitude.parse::<f64>().unwrap(), locator.longitude.parse::<f64>().unwrap());
        let target_locator = Location::new(target_locator.latitude.parse::<f64>().unwrap(), target_locator.longitude.parse::<f64>().unwrap());
    
        let distance = locator.distance_to(&target_locator).unwrap();
        distance.meters()
    }

    // This function maps the request domain name to the CDN server
    pub fn map_request_to_cdn(&self, domain: &str) -> String {
        domain.to_string()
    }

    // This function gets a sorted list of distance from the client to the CDN servers in ascending order
    pub async fn get_sorted_cdn_servers(&self, client_ip: &str) -> Vec<(f64, String)> {
        let mut cdn_servers = vec![];
        // Get the distance from the client to each CDN server
        for (cdn_ip, _) in self.cdn_server.iter() {
            let distance = self.get_distance_from_ip(client_ip, cdn_ip).await;
            cdn_servers.push((distance, cdn_ip.to_string()));
        }
        cdn_servers.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        cdn_servers
    }

    
}
