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
    let mut dns_server = DnsServer::new(&port);
    // generate fake cdn servers in cdn_server: HashMap::new()
    // println!("CDN servers: {:?}", dns_server.socket);
    // dbg!(DnsServer::get_cache("cdn-http3.khoury.northeastern.edu", "20131").await);
    dns_server.start().await;
}
