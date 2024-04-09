use ipgeolocate::{GeoError, Locator, Service};
use geoutils::Location;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::net::UdpSocket;
use bytes::{Bytes, BytesMut};
use dns_message_parser::{Dns, Flags, Opcode, RCode};
use dns_message_parser::rr::{A, RR};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::Ipv4Addr;
use std::time::Instant;

pub struct DnsServer {
    // Hashmap to store the CDN IP address and information
    cdn_server: HashMap<String, CdnServerInfo>,
    // Number of available CDN servers
    available_cdn_count : i32,
    // UDP socket
    socket: UdpSocket,
    cache: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    cpu_usage: Arc<Mutex<HashMap<String, f32>>>,
    cdn_port: String,
    distance_to_origin: HashMap<String, f64>,
    client_distance_cache: HashMap<String, HashMap<String, f64>>
}

struct CdnServerInfo {
    domain_name: String,
    geolocation: Locator,
}

impl DnsServer {
    // This function is used to create a new instance of the DnsServer struct
    pub fn new(port: &str) -> Self {
        let dns_server = DnsServer {
            cdn_server: HashMap::new(),
            available_cdn_count: 7,
            socket: UdpSocket::bind(format!("0.0.0.0:{port}")).unwrap(), // bind to 0.0.0.0 so that it can listen on all available ip addresses on the machine
            cache: Arc::new(Mutex::new(HashMap::new())),
            cpu_usage: Arc::new(Mutex::new(HashMap::new())),
            cdn_port: port.to_string(),
            distance_to_origin: HashMap::new(),
            client_distance_cache: HashMap::new()
        };
        dns_server
    }

    // This function is used to init a CDN server information to the cdn_server hashmap
    pub async fn init_cdn_geolocation(&mut self) {
        let mut cpu = self.cpu_usage.lock().await;
        // Save all the ip addresses of the CDN servers
        self.cdn_server.insert("45.33.55.171".to_string(), CdnServerInfo {
            domain_name: "cdn-http3.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("45.33.55.171").await.unwrap(),
        }); // cdn-http3.khoury.northeastern.edu
        cpu.insert("45.33.55.171".to_string(), 0_f32);
        self.distance_to_origin.insert("45.33.55.171".to_string(), 4320779.177);

        self.cdn_server.insert("170.187.142.220".to_string(), CdnServerInfo {
            domain_name: "cdn-http4.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("170.187.142.220").await.unwrap(),
        }); // cdn-http4.khoury.northeastern.edu
        cpu.insert("170.187.142.220".to_string(), 0_f32);
        self.distance_to_origin.insert("170.187.142.220".to_string(), 1511283.111);

        self.cdn_server.insert("213.168.249.157".to_string(), CdnServerInfo {
            domain_name: "cdn-http7.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("213.168.249.157").await.unwrap(),
        }); // cdn-http7.khoury.northeastern.edu
        cpu.insert("213.168.249.157".to_string(), 0_f32);
        self.distance_to_origin.insert("213.168.249.157".to_string(), 5275439.248);

        self.cdn_server.insert("139.162.82.207".to_string(), CdnServerInfo {
            domain_name: "cdn-http11.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("139.162.82.207").await.unwrap(),
        }); // cdn-http11.khoury.northeastern.edu
        cpu.insert("139.162.82.207".to_string(), 0_f32);
        self.distance_to_origin.insert("139.162.82.207".to_string(), 10812481.474);

        self.cdn_server.insert("45.79.124.209".to_string(), CdnServerInfo {
            domain_name: "cdn-http14.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("45.79.124.209").await.unwrap(),
        }); // cdn-http14.khoury.northeastern.edu
        cpu.insert("45.79.124.209".to_string(), 0_f32);
        self.distance_to_origin.insert("45.79.124.209".to_string(), 12261839.872);

        self.cdn_server.insert("192.53.123.145".to_string(), CdnServerInfo {
            domain_name: "cdn-http15.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("192.53.123.145").await.unwrap(),
        }); // cdn-http15.khoury.northeastern.edu
        cpu.insert("192.53.123.145".to_string(), 0_f32);
        self.distance_to_origin.insert("192.53.123.145".to_string(), 696856.872);

        self.cdn_server.insert("192.46.221.203".to_string(), CdnServerInfo {
            domain_name: "cdn-http16.khoury.northeastern.edu".to_string(),
            geolocation: self.get_geolocation("192.46.221.203").await.unwrap(),
        }); // cdn-http16.khoury.northeastern.edu
        cpu.insert("192.46.221.203".to_string(), 0_f32);
        self.distance_to_origin.insert("192.46.221.203".to_string(), 16241736.48);
    }

    // This function will start the DNS server
    pub async fn start(&mut self) {

        // Get the geo location of the CDN servers
        self.init_cdn_geolocation().await;

        for (ip, cdn_server)  in self.cdn_server.iter() {
            let cache_ptr = Arc::clone(&self.cache);
            let cpu_usage_ptr = Arc::clone(&self.cpu_usage);
            let port = self.cdn_port.clone();
            let domain = cdn_server.domain_name.clone();
            let copy_ip = ip.to_string();
            // let mut timestamp = Instant::now();

            tokio::spawn(async move {
                
                loop {
                    // let cur_time = Instant::now();
                    // if cur_time.duration_since(timestamp).as_secs() > 2 {
                        // match DnsServer::get_cache(domain.clone(), port.clone()).await {
                        //     Ok(res) => {
                        //         timestamp = Instant::now();
                        //     }
                        //     Err(_) => {}
                        // }
                    // }
                    match DnsServer::get_cache(domain.clone(), port.clone()).await {
                        Ok(res) => {
                            let res_vec: Vec<&str> = res.split(" ").collect();
                            let mut new_set: HashSet<String> = HashSet::new();

                            for cache_content in res_vec[..res_vec.len() - 1].into_iter() {
                                new_set.insert(cache_content.to_string());
                            }

                            let mut cache = cache_ptr.lock().await;
                            cache.insert(copy_ip.clone(), new_set);
                            drop(cache);

                            let mut cpu_usage = cpu_usage_ptr.lock().await;
                            let old_usage = match cpu_usage.get(&copy_ip) {
                                Some(x) => { *x }
                                None => { 0_f32 }
                            };
                            cpu_usage.insert(copy_ip.clone(), match res_vec[res_vec.len() - 1].parse::<f32>() {
                                Ok(usage) => { usage }
                                Err(_) => { old_usage }
                            });
                            drop(cpu_usage);
                        }
                        Err(_) => {}
                    }

                    let test_usage = cache_ptr.lock().await;

                    let test = test_usage.get(&copy_ip);
                    dbg!(test, &domain);
                    drop(test_usage);

                   

                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            });

        }

        

        loop {
            // If already send all the IP once -> set all to false
            // if (self.available_cdn_count == 0) {
            //     for (cdn_ip, cdn_info) in self.cdn_server.iter_mut() {
            //         cdn_info.available = true;
            //     }
            //     self.available_cdn_count = 7;
            // }
    
            // Read the message from the udp socket
            let (client_address, dns_question) = self.get_question_domain_name();
            // String of client address, for sending response
            let client_address_str = client_address.to_string();
            // Remove port number from the source address
            let client_ip = client_address_str.split(":").collect::<Vec<&str>>()[0];
    
            // Get the sorted list of CDN servers based on the distance from the client
            let sorted_cdn_servers = self.get_sorted_cdn_servers(&client_ip, "Wiki").await;
    
            // Get the closest CDN server that is available
            let mut closest_cdn_server = sorted_cdn_servers[0].1.as_ref();
            // for (distance, cdn_ip) in &sorted_cdn_servers {
            //     if (self.cdn_server.get(cdn_ip).unwrap().available) {
            //         // Get the closest CDN server that is available
            //         closest_cdn_server = cdn_ip;
            //         // Set the CDN server to unavailable
            //         self.cdn_server.get_mut(cdn_ip).unwrap().available = false;
            //         self.available_cdn_count -= 1;
            //         break;
            //     }
            // }
            dbg!(&sorted_cdn_servers);
            
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
        // println!("domain_name: {:?}", domain_name);
        // // Read until second to the last character to remove the last dot
        // let domain_name = &domain_name[..domain_name.len() - 1];
        let q = dns.questions.clone();
        println!("Received request for domain: {:?}", q[q.len() - 1]);
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
    async fn get_distance_from_ip(&self, locator: &Locator, target_locator: &Locator) -> f64 {
        // Calculate the distance between the two IP addresses
        let locator = Location::new(locator.latitude.parse::<f64>().unwrap(), locator.longitude.parse::<f64>().unwrap());
        let target_locator = Location::new(target_locator.latitude.parse::<f64>().unwrap(), target_locator.longitude.parse::<f64>().unwrap());
    
        let distance = locator.distance_to(&target_locator).unwrap();
        distance.meters()
    }

    // This function gets a sorted list of distance from the client to the CDN servers in ascending order
    async fn get_sorted_cdn_servers(&mut self, client_ip: &str, content: &str) -> Vec<(f64, String)> {
        let mut cdn_servers = vec![];
        // Get client ip geolocation
        let client_ip_geolocation = self.get_geolocation(client_ip).await.unwrap();

        let mut client_to_server: HashMap<String, f64> = HashMap::new();

        if self.client_distance_cache.contains_key(client_ip) {
            client_to_server = self.client_distance_cache.get(client_ip).unwrap().clone();
        } else {
            for cdn_ip in self.cdn_server.keys() {
                let distance = self.get_distance_from_ip(&client_ip_geolocation, &self.cdn_server.get(cdn_ip).unwrap().geolocation).await;
                client_to_server.insert(cdn_ip.clone(), distance);
            }

            self.client_distance_cache.insert(client_ip.to_string(), client_to_server.clone());
        }

        // Get the distance from the client to each CDN server
        for (cdn_ip, _) in self.cdn_server.iter() {
            let cpu_usage = self.cpu_usage.lock().await;
            let usage = *cpu_usage.get(cdn_ip).unwrap();
            drop(cpu_usage);

            if usage > 90_f32 {
                continue
            }

            let cache = self.cache.lock().await;
            let cache_set = cache.get(cdn_ip).unwrap().clone();
            drop(cache);

            let mut distance = *client_to_server.get(cdn_ip).unwrap();
            if !cache_set.contains(content) {
                distance += *self.distance_to_origin.get(cdn_ip).unwrap();
            }

            cdn_servers.push((distance, cdn_ip.to_string()));
        }

        if cdn_servers.is_empty() {
            cdn_servers.push((0_f64, "192.53.123.145".to_string()))
        } else {
            cdn_servers.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }

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
        let domain_name = self.cdn_server.get(closest_cdn_server).unwrap().domain_name.to_string().parse().unwrap();
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

    pub async fn get_cache(domain: String, port: String) -> Result<String, ()> {
        let client = reqwest::Client::new();

        match client.get(&format!("http://{}:{}/api/getCache", domain, port)).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => { Ok(text) }
                    Err(_) => {
                        dbg!(format!("Error: can't parse the content of cache records from {}", domain));
                        Err(())
                    }
                }
            }
            Err(_) => {
                dbg!(format!("Error: can't get cache from {}", domain));
                Err(())
            }
        }

    }

    
}
