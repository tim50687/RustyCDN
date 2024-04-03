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

    // test get sorted cdn servers
    let mut dns_server = DnsServer::new();
    // generate fake cdn servers in cdn_server: HashMap::new()
    dns_server.cdn_server.insert("132.21.2.3".to_string(), true);
    dns_server.cdn_server.insert("113.20.2.3".to_string(), true);
    dns_server.cdn_server.insert("109.21.1.3".to_string(), true);
    println!("CDN servers: {:?}", dns_server.cdn_server);
    let client_ip = "1.1.1.1";
    let cdn_servers = dns_server.get_sorted_cdn_servers(client_ip).await;
    println!("CDN servers sorted by distance: {:?}", cdn_servers);
}
