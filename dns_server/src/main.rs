mod utils;

use utils::parse_arguments;


fn main() {
    // Get the port number and CDN server name from the command line arguments
    let matches = parse_arguments();
    let port = matches.get_one::<String>("port").unwrap();
    let cdn = matches.get_one::<String>("cdn").unwrap();
    println!("Port: {}", port);
    println!("CDN: {}", cdn);
}
