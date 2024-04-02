mod utils;
mod dns_server;

use utils::parse_arguments;
use dns_server::DnsServer;

#[tokio::main]
async fn main() {
    // Get the port number and CDN server name from the command line arguments
    let matches = parse_arguments();
    let port = matches.get_one::<String>("port").unwrap();
    let cdn = matches.get_one::<String>("cdn").unwrap();
    println!("Port: {}", port);
    println!("CDN: {}", cdn);

    // Get the geolocation of an IP address
    let dns_server = DnsServer::new();
    let ip = "1.1.1.1";
    let ip2 = "128.0.9.1";
    let distance = dns_server.get_distance_from_ip(ip, ip2).await;
    println!("Distance between {} and {} is {} meters", ip, ip2, distance);
}
