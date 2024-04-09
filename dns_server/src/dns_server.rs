use bytes::{Bytes, BytesMut};
use dns_message_parser::rr::{A, RR};
use dns_message_parser::{Dns, Flags, Opcode, RCode};
use geoutils::Location;
use ipgeolocate::{GeoError, Locator, Service};
use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::Mutex;


pub struct DnsServer {
    // Hashmap to store the CDN IP address and information
    cdn_server: HashMap<String, CdnServerInfo>,
    // UDP socket
    socket: UdpSocket,
    // cache: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    cpu_usage: Arc<Mutex<HashMap<String, f32>>>,
    cdn_port: String,
    distance_to_origin: HashMap<String, f64>,
    client_distance_cache: Arc<Mutex<HashMap<String, HashMap<String, f64>>>>,
    availability: Arc<Mutex<HashMap<String, bool>>>,
    location: Location
}

#[derive(Clone)]
struct CdnServerInfo {
    domain_name: String,
    geolocation: Location,
}

impl DnsServer {
    // This function is used to create a new instance of the DnsServer struct
    pub fn new(port: &str) -> Self {
        let mut availability: HashMap<String, bool> = HashMap::new();
        availability.insert("45.33.55.171".to_string(), true);
        availability.insert("170.187.142.220".to_string(), true);
        availability.insert("213.168.249.157".to_string(), true);
        availability.insert("139.162.82.207".to_string(), true);
        availability.insert("45.79.124.209".to_string(), true);
        availability.insert("192.53.123.145".to_string(), true);
        availability.insert("192.46.221.203".to_string(), true);

        let dns_server = DnsServer {
            cdn_server: HashMap::new(),
            socket: UdpSocket::bind(format!("0.0.0.0:{port}")).unwrap(), // bind to 0.0.0.0 so that it can listen on all available ip addresses on the machine
            // cache: Arc::new(Mutex::new(HashMap::new())),
            cpu_usage: Arc::new(Mutex::new(HashMap::new())),
            cdn_port: port.to_string(),
            distance_to_origin: HashMap::new(),
            client_distance_cache: Arc::new(Mutex::new(HashMap::new())),
            availability: Arc::new(Mutex::new(availability)),
            location: Location::new(40.8229, -74.4592),
        };
        dns_server
    }

    // This function is used to init a CDN server information to the cdn_server hashmap
    pub async fn init_cdn_geolocation(&mut self) {
        let mut cpu = self.cpu_usage.lock().await;
        // Save all the ip addresses of the CDN servers
        self.cdn_server.insert(
            "45.33.55.171".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http3.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(37.5625, -122.0004),
            },
        ); // cdn-http3.khoury.northeastern.edu
        cpu.insert("45.33.55.171".to_string(), 0_f32);
        self.distance_to_origin
            .insert("45.33.55.171".to_string(), 4320779.177);

        self.cdn_server.insert(
            "170.187.142.220".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http4.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(33.7485, -84.3871),
            },
        ); // cdn-http4.khoury.northeastern.edu
        cpu.insert("170.187.142.220".to_string(), 0_f32);
        self.distance_to_origin
            .insert("170.187.142.220".to_string(), 1511283.111);

        self.cdn_server.insert(
            "213.168.249.157".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http7.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(51.5074, -0.1196),
            },
        ); // cdn-http7.khoury.northeastern.edu
        cpu.insert("213.168.249.157".to_string(), 0_f32);
        self.distance_to_origin
            .insert("213.168.249.157".to_string(), 5275439.248);

        self.cdn_server.insert(
            "139.162.82.207".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http11.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(35.6893, 139.6899),
            },
        ); // cdn-http11.khoury.northeastern.edu
        cpu.insert("139.162.82.207".to_string(), 0_f32);
        self.distance_to_origin
            .insert("139.162.82.207".to_string(), 10812481.474);

        self.cdn_server.insert(
            "45.79.124.209".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http14.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(19.0748, 72.8856),
            },
        ); // cdn-http14.khoury.northeastern.edu
        cpu.insert("45.79.124.209".to_string(), 0_f32);
        self.distance_to_origin
            .insert("45.79.124.209".to_string(), 12261839.872);

        self.cdn_server.insert(
            "192.53.123.145".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http15.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(43.709, -79.4057),
            },
        ); // cdn-http15.khoury.northeastern.edu
        cpu.insert("192.53.123.145".to_string(), 0_f32);
        self.distance_to_origin
            .insert("192.53.123.145".to_string(), 696856.872);

        self.cdn_server.insert(
            "192.46.221.203".to_string(),
            CdnServerInfo {
                domain_name: "cdn-http16.khoury.northeastern.edu".to_string(),
                geolocation: Location::new(-33.8715, 151.2006),
            },
        ); // cdn-http16.khoury.northeastern.edu
        cpu.insert("192.46.221.203".to_string(), 0_f32);
        self.distance_to_origin
            .insert("192.46.221.203".to_string(), 16241736.48);
    }

    // This function will start the DNS server
    pub async fn start(&mut self) {
        // Get the geo location of the CDN servers
        self.init_cdn_geolocation().await;

        for (ip, cdn_server) in self.cdn_server.iter() {
            // let cache_ptr = Arc::clone(&self.cache);
            let cpu_usage_ptr = Arc::clone(&self.cpu_usage);
            let port = self.cdn_port.clone();
            let domain = cdn_server.domain_name.clone();
            let copy_ip = ip.to_string();
            let availability_ptr = Arc::clone(&self.availability);

            tokio::spawn(async move {
                loop {
                    match DnsServer::get_usage(domain.clone(), port.clone()).await {
                        Ok(res) => {
                            let mut availability = availability_ptr.lock().await;
                            let handle = availability.get_mut(&copy_ip).unwrap();
                            *handle = true;
                            drop(availability);

                            let mut cpu_usage = cpu_usage_ptr.lock().await;
                            let old_usage = match cpu_usage.get(&copy_ip) {
                                Some(x) => *x,
                                None => 0_f32,
                            };
                            cpu_usage.insert(
                                copy_ip.clone(),
                                match res.trim().parse::<f32>() {
                                    Ok(usage) => usage,
                                    Err(_) => old_usage,
                                },
                            );
                            drop(cpu_usage);
                        }
                        Err(_) => {
                            let mut availability = availability_ptr.lock().await;
                            let handle = availability.get_mut(&copy_ip).unwrap();
                            *handle = false;
                            drop(availability);
                        }
                    }

                    let test_usage = cpu_usage_ptr.lock().await;

                    let test = *test_usage.get(&copy_ip).unwrap();
                    drop(test_usage);
                    // dbg!(test, &domain);

                    tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
                }
            });
        }

        loop {
            // Read the message from the udp socket
            let (client_address, dns_question) = self.get_question_domain_name();

            let mut cloned = self.clone();

            tokio::spawn(async move {
                // String of client address, for sending response
                let client_address_str = client_address.to_string();
                // Remove port number from the source address
                let client_ip = client_address_str.split(":").collect::<Vec<&str>>()[0];
                let sorted_cdn_servers = cloned.get_sorted_cdn_servers(&client_ip, " ").await;
                let ans;

                if sorted_cdn_servers.is_empty() {
                    ans = cloned.generate_response_when_all_cdnservers_down(
                        &dns_question,
                        "3.129.217.143",
                        "ec2-3-129-217-143.us-east-2.compute.amazonaws.com",
                    )
                } else {
                    let mut closest_cdn_server: &str = sorted_cdn_servers[0].1.as_ref();
                    ans = cloned.generate_response(&dns_question, closest_cdn_server);
                }

                dbg!(&sorted_cdn_servers);
                dbg!(&client_address);

                cloned.socket.send_to(&ans, &client_address).unwrap();
            });

            // // String of client address, for sending response
            // let client_address_str = client_address.to_string();
            // // Remove port number from the source address
            // let client_ip = client_address_str.split(":").collect::<Vec<&str>>()[0];

            // Get the sorted list of CDN servers based on the distance from the client
            // let sorted_cdn_servers = self.get_sorted_cdn_servers(&client_ip, " ").await;

            // let ans;

            // if sorted_cdn_servers.is_empty() {
            //     ans = self.generate_response_when_all_cdnservers_down(
            //         &dns_question,
            //         "3.129.217.143",
            //         "ec2-3-129-217-143.us-east-2.compute.amazonaws.com",
            //     )
            // } else {
            //     let mut closest_cdn_server: &str = sorted_cdn_servers[0].1.as_ref();
            //     ans = self.generate_response(&dns_question, closest_cdn_server);
            // }

            // dbg!(&sorted_cdn_servers);

            // self.socket.send_to(&ans, &client_address).unwrap();
        }
    }

    pub fn clone(&self) -> Self {
        let cloned = DnsServer {
            cdn_server: self.cdn_server.clone(),
            socket: self.socket.try_clone().unwrap(), // bind to 0.0.0.0 so that it can listen on all available ip addresses on the machine
            // cache: Arc::new(Mutex::new(HashMap::new())),
            cpu_usage: Arc::clone(&self.cpu_usage),
            cdn_port: self.cdn_port.clone(),
            distance_to_origin: HashMap::new(),
            client_distance_cache: Arc::clone(&self.client_distance_cache),
            availability: Arc::clone(&self.availability),
            location: Location::new(40.8229, -74.4592),
        };

        cloned
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
    async fn get_geolocation(&self, ip: &str) -> Result<Locator, GeoError> {
        let service = Service::IpApi;
        let backup_service = Service::FreeGeoIp;

        let locator = match Locator::get(ip, service).await {
            Ok(locator) => locator,
            Err(_) => match Locator::get(ip, backup_service).await {
                Ok(locator2) => locator2,
                Err(e) => return Err(e),
            },
        };
        Ok(locator)
    }

    // This function is used to get the distance between two IP addresses
    async fn get_distance_from_ip(&self, location: &Location, target_location: &Location) -> f64 {
        // Calculate the distance between the two IP addresses
        // let location = Location::new(locator.latitude.parse::<f64>().unwrap(), locator.longitude.parse::<f64>().unwrap());
        // let target_locator = Location::new(target_locator.latitude.parse::<f64>().unwrap(), target_locator.longitude.parse::<f64>().unwrap());

        let distance = location.distance_to(&target_location).unwrap();
        distance.meters()
    }

    // This function gets a sorted list of distance from the client to the CDN servers in ascending order
    async fn get_sorted_cdn_servers(
        &mut self,
        client_ip: &str,
        content: &str,
    ) -> Vec<(f64, String)> {
        let mut cdn_servers = vec![];
        // Get client ip geolocation
        let mut client_ip_geolocation = self.location.clone();

        match self.get_geolocation(client_ip).await {
            Ok(client_ip_geolocator) => {
                client_ip_geolocation = Location::new(
                    client_ip_geolocator.latitude.parse::<f64>().unwrap(),
                    client_ip_geolocator.longitude.parse::<f64>().unwrap(),
                );
            }
            Err(_) => {}
        }

        let mut client_to_server: HashMap<String, f64> = HashMap::new();

        let mut d_cache = self.client_distance_cache.lock().await;

        if d_cache.contains_key(client_ip) {
            client_to_server = d_cache.get(client_ip).unwrap().clone();
        } else {
            for cdn_ip in self.cdn_server.keys() {
                let distance = self
                    .get_distance_from_ip(
                        &client_ip_geolocation,
                        &self.cdn_server.get(cdn_ip).unwrap().geolocation,
                    )
                    .await;
                client_to_server.insert(cdn_ip.clone(), distance);
            }
            d_cache.insert(client_ip.to_string(), client_to_server.clone());
        }
        drop(d_cache);

        // Get the distance from the client to each CDN server
        for (cdn_ip, _) in self.cdn_server.iter() {
            let availability = self.availability.lock().await;
            let ava = *availability.get(cdn_ip).unwrap();
            drop(availability);
            if !ava {
                continue;
            }

            let cpu_usage = self.cpu_usage.lock().await;
            let usage = *cpu_usage.get(cdn_ip).unwrap();
            drop(cpu_usage);

            if usage > 90_f32 {
                continue;
            }

            // let cache = self.cache.lock().await;
            // let cache_set = cache.get(cdn_ip).unwrap().clone();
            // drop(cache);

            let distance = *client_to_server.get(cdn_ip).unwrap();
            // if !cache_set.contains(content) {
            //     distance += *self.distance_to_origin.get(cdn_ip).unwrap();
            // }

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
        let domain_name = self
            .cdn_server
            .get(closest_cdn_server)
            .unwrap()
            .domain_name
            .to_string()
            .parse()
            .unwrap();
        let mut answer = Vec::new();
        // Turn string into ipv4 address
        let ip_vec = closest_cdn_server
            .split(".")
            .map(|x| x.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        let ipv4_addr = Ipv4Addr::new(ip_vec[0], ip_vec[1], ip_vec[2], ip_vec[3]);
        answer.push(RR::A(A {
            domain_name,
            ttl: 0,
            ipv4_addr,
        }));

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

    fn generate_response_when_all_cdnservers_down(
        &self,
        dns_question: &Dns,
        closest_cdn_server: &str,
        domain_name: &str,
    ) -> BytesMut {
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
        let domain_name = domain_name.to_string().parse().unwrap();
        let mut answer = Vec::new();
        // Turn string into ipv4 address
        dbg!(&closest_cdn_server);
        let ip_vec = closest_cdn_server
            .split(".")
            .map(|x| x.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        let ipv4_addr = Ipv4Addr::new(ip_vec[0], ip_vec[1], ip_vec[2], ip_vec[3]);
        answer.push(RR::A(A {
            domain_name,
            ttl: 0,
            ipv4_addr,
        }));

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

    pub async fn get_usage(domain: String, port: String) -> Result<String, ()> {
        let client = reqwest::Client::new();

        match client
            .get(&format!("http://{}:{}/api/getUsage", domain, port))
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(text) => Ok(text),
                Err(_) => {
                    dbg!(format!(
                        "Error: can't parse the content of cache records from {}",
                        domain
                    ));
                    Err(())
                }
            },
            Err(_) => {
                dbg!(format!("Error: can't get cache from {}", domain));
                Err(())
            }
        }
    }
}
