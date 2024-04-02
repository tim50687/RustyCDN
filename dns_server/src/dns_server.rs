use ipgeolocate::{GeoError, Locator, Service};
use geoutils::Location;
// This function is used to get the geolocation of an IP address
pub async fn get_geolocation(ip: &str) -> Result<Locator, GeoError>{
    let service = Service::IpApi;
    let locator = match Locator::get(ip, service).await {
        Ok(locator) => locator,
        Err(error) => return Err(error),
    };
    Ok(locator)
}

// This function is used to get the distance between two IP addresses
pub async fn get_distance_from_ip(ip: &str, target_ip: &str) -> f64 {
    let locator = get_geolocation(ip).await.unwrap();
    let target_locator = get_geolocation(target_ip).await.unwrap();
    // Calculate the distance between the two IP addresses
    let locator = Location::new(locator.latitude.parse::<f64>().unwrap(), locator.longitude.parse::<f64>().unwrap());
    let target_locator = Location::new(target_locator.latitude.parse::<f64>().unwrap(), target_locator.longitude.parse::<f64>().unwrap());

    let distance = locator.distance_to(&target_locator).unwrap();
    distance.meters()
}