use ipgeolocate::{GeoError, Locator, Service};
use geoutils::Location;
use std::{collections::HashMap};
use std::net::UdpSocket;
use bytes::{Bytes, BytesMut};
use dns_message_parser::{Dns, DomainName, Flags, Opcode, RCode};
use dns_message_parser::question::{QClass, QType, Question};
use dns_message_parser::rr::{A, RR};
use std::net::Ipv4Addr;
pub struct DnsServer {
    client_ip: String,
    dns_port: String,
    // hashmap to store the CDN IP address and availability
    pub cdn_server: HashMap<String, bool>,
    cdn_ip_domain_map: HashMap<String, String>,
    available_cdn_count : i32,
    // UDP socket
    pub socket: UdpSocket,
}

impl DnsServer {
    // This function is used to create a new instance of the DnsServer struct
    pub fn new(port: &str) -> Self {
        let mut dns_server = DnsServer {
            client_ip: format!("127.0.0.1:{port}"),
            dns_port: port.to_string(),
            cdn_server: HashMap::new(),
            cdn_ip_domain_map: HashMap::new(),
            available_cdn_count: 7,
            socket: UdpSocket::bind(format!("127.0.0.1:{port}")).unwrap(),
        };
        // Save all the ip addresses of the CDN servers
        dns_server.cdn_server.insert("45.33.55.171".to_string(), true); // cdn-http3.khoury.northeastern.edu
        dns_server.cdn_server.insert("170.187.142.220".to_string(), true); // cdn-http4.khoury.northeastern.edu
        dns_server.cdn_server.insert("213.168.249.157".to_string(), true); // cdn-http7.khoury.northeastern.edu
        dns_server.cdn_server.insert("139.162.82.207".to_string(), true); // cdn-http11.khoury.northeastern.edu
        dns_server.cdn_server.insert("45.79.124.209".to_string(), true); // cdn-http14.khoury.northeastern.edu
        dns_server.cdn_server.insert("192.53.123.145".to_string(), true); // cdn-http15.khoury.northeastern.edu
        dns_server.cdn_server.insert("192.46.221.203".to_string(), true); // cdn-http16.khoury.northeastern.edu

        // Save the domain name of the CDN servers
        dns_server.cdn_ip_domain_map.insert("45.33.55.171".to_string(), "cdn-http3.khoury.northeastern.edu".to_string()); 
        dns_server.cdn_ip_domain_map.insert("170.187.142.220".to_string(), "cdn-http4.khoury.northeastern.edu".to_string()); 
        dns_server.cdn_ip_domain_map.insert("213.168.249.157".to_string(), "cdn-http7.khoury.northeastern.edu".to_string()); 
        dns_server.cdn_ip_domain_map.insert("139.162.82.207".to_string(), "cdn-http11.khoury.northeastern.edu".to_string()); 
        dns_server.cdn_ip_domain_map.insert("45.79.124.209".to_string(), "cdn-http14.khoury.northeastern.edu".to_string()); 
        dns_server.cdn_ip_domain_map.insert("192.53.123.145".to_string(), "cdn-http15.khoury.northeastern.edu".to_string()); 
        dns_server.cdn_ip_domain_map.insert("192.46.221.203".to_string(), "cdn-http16.khoury.northeastern.edu".to_string()); 
        dns_server
    }

    // This function will start the DNS server
    pub async fn start(&mut self) {

        loop {
            // If already send all the IP once -> set all to false
            if (self.available_cdn_count == 0) {
                for (cdn_ip, used) in self.cdn_server.iter_mut() {
                    *used = false;
                }
                self.available_cdn_count = 7;
            }
    
            // Read the message from the udp socket
            let (client_address, dns_question) = self.get_question_domain_name();
            // Remove port number from the source address
            let client_address_str = client_address.to_string();
            let client_ip = client_address_str.split(":").collect::<Vec<&str>>()[0];
            // for testing
            let client_ip = "8.8.8.8"; 
    
            // Get the sorted list of CDN servers based on the distance from the client
            let sorted_cdn_servers = self.get_sorted_cdn_servers(&client_ip).await;
    
            // Get the closest CDN server that is available
            let mut closest_cdn_server = "";
            for (distance, cdn_ip) in &sorted_cdn_servers {
                if (*self.cdn_server.get(cdn_ip).unwrap()) {
                    // Get the closest CDN server that is available
                    closest_cdn_server = cdn_ip;
                    // Set the CDN server to unavailable
                    *self.cdn_server.get_mut(cdn_ip).unwrap() = false;
                    break;
                }
            }
            
            let ans = self.generate_response(&dns_question, closest_cdn_server);
            // Send the response to the client
            self.socket.send_to(&ans, &client_address).unwrap();
        }
    }

    // This function will read from the request and get the dns question and src ip
    pub fn get_question_domain_name(&self) -> (String, Dns) {
        // Read the message from the udp socket
        let mut buf = [0; 1024];
        let (amt, src) = self.socket.recv_from(&mut buf).unwrap();
        // // Use dns parser to decode the message
        let bytes = Bytes::copy_from_slice(&buf[..amt]);
        let dns = Dns::decode(bytes).unwrap();
        // Get the domain name of the dns question
        // let domain_name = &dns.questions[0].domain_name.to_string();
        // // Read until second to the last character to remove the last dot
        // let domain_name = &domain_name[..domain_name.len() - 1];
        // println!("Received request for domain: {:?}", dns);
        // println!("Received request from: {:?}", src);
        
        // println!("Received request from: {:?}", src);
        (src.to_string(), dns)
    }

    // This function is used to get the geolocation of an IP address
    async fn get_geolocation(&self,ip: &str) -> Result<Locator, GeoError>{
        let service = Service::IpApi;

        let locator = match Locator::get(ip, service).await {
            Ok(locator) => locator,
            Err(error) => return Err(error),
        };
        Ok(locator)
    }
    
    // This function is used to get the distance between two IP addresses
    async fn get_distance_from_ip(&self, ip: &str, target_ip: &str) -> f64 {
        let locator = self.get_geolocation(ip).await.unwrap();
        let target_locator = self.get_geolocation(target_ip).await.unwrap();

        // Calculate the distance between the two IP addresses
        let locator = Location::new(locator.latitude.parse::<f64>().unwrap(), locator.longitude.parse::<f64>().unwrap());
        let target_locator = Location::new(target_locator.latitude.parse::<f64>().unwrap(), target_locator.longitude.parse::<f64>().unwrap());
    
        let distance = locator.distance_to(&target_locator).unwrap();
        distance.meters()
    }

    // This function gets a sorted list of distance from the client to the CDN servers in ascending order
    async fn get_sorted_cdn_servers(&self, client_ip: &str) -> Vec<(f64, String)> {
        let mut cdn_servers = vec![];
        // Get the distance from the client to each CDN server
        for (cdn_ip, _) in self.cdn_server.iter() {
            let distance = self.get_distance_from_ip(client_ip, cdn_ip).await;
            cdn_servers.push((distance, cdn_ip.to_string()));
        }
        cdn_servers.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        cdn_servers
    }

    // This function will generate DNS response
    pub fn generate_response(&self, dns_question: &Dns, closest_cdn_server: &str) -> BytesMut {
        // Create a new DNS response
        // Fill out the fields of the DNS response
        let id = dns_question.id;
        let flags = Flags {
            qr: true,
            opcode: Opcode::Query,
            aa: false,
            tc: false,
            rd: true,
            ra: false,
            ad: false,
            cd: false,
            rcode: RCode::NoError,
        };
        let questions = dns_question.questions.clone();
        let authorities = vec![];
        let additionals = vec![];
        // Add the CDN server IP address to the answer
        let domain_name = self.cdn_ip_domain_map.get(closest_cdn_server).unwrap().to_string().parse().unwrap();
        let mut answer = Vec::new();
        // Turn string into ipv4 address
        let ip_vec = closest_cdn_server.split(".").map(|x| x.parse::<u8>().unwrap()).collect::<Vec<u8>>();
        let ipv4_addr = Ipv4Addr::new(ip_vec[0], ip_vec[1], ip_vec[2], ip_vec[3]);
        answer.push(RR::A(A { domain_name , ttl: 0,ipv4_addr }));

        let dns_response = Dns {
            id,
            flags,
            questions,
            answers: answer,
            authorities,
            additionals,
        };
        
        // Encode the DNS response
        let response = dns_response.encode().unwrap();

        response
    }

    
}
