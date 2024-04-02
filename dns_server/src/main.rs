mod utils;
mod dns_utils;

use utils::parse_arguments;
use dns_utils::{get_geolocation, get_distance_from_ip};

#[tokio::main]
async fn main() {
    // Get the port number and CDN server name from the command line arguments
    let matches = parse_arguments();
    let port = matches.get_one::<String>("port").unwrap();
    let cdn = matches.get_one::<String>("cdn").unwrap();
    println!("Port: {}", port);
    println!("CDN: {}", cdn);

    // Get the geolocation of an IP address
    let ip = "1.1.1.1";
    let ip2 = "128.0.9.1";
    let distance = get_distance_from_ip(ip, ip2).await;
    println!("Distance between {} and {} is {} meters", ip, ip2, distance);
}
