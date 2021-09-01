use serde_json::Value;
use thiserror::Error;


/// The main struct for getting geocoding data.
#[derive(Clone, PartialEq, Debug)]
pub struct Nominatim {
    pub latitude: f64,
    pub longitude: f64,
    pub location: String,
    pub place_id: usize,
    pub osm_id: usize,
    /// Address is only available on search.
    pub address: Option<Address>,
}

impl Nominatim {
    /// Get data from an openstreetmap ID.
    pub async fn lookup<T: AsRef<str>>(osm_id: T) -> Result<Self, NominatimError> {
        let uri = &format!(
            "https://nominatim.openstreetmap.org/lookup?osm_ids={}&format=json",
            osm_id.as_ref().replace(" ", "")
        );

        let geocode = match surf::get(uri).recv_string().await {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Http(error.to_string())),
        };

        let geocode_json: Value = match serde_json::from_str(&geocode) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        return Self::parse(&geocode_json[0]);
    }

    /// Get data from the name of a location.
    pub async fn search<T: AsRef<str>>(name: T) -> Result<Self, NominatimError> {
        let uri = &format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json",
            name.as_ref().replace(" ", "+")
        );

        let geocode = match surf::get(uri).recv_string().await {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Http(error.to_string())),
        };

        let geocode_json: Value = match serde_json::from_str(&geocode) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        return Self::parse(&geocode_json[0]);
    }

    /// Get data from the coordinates of a location.
    pub async fn reverse(lat: f64, lon: f64) -> Result<Self, NominatimError> {
        let uri = &format!(
            "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json",
            lat, lon
        );

        let geocode = match surf::get(uri).recv_string().await {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Http(error.to_string())),
        };

        let geocode_json: Value = match serde_json::from_str(&geocode) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        return Self::parse(&geocode_json);
    }

    /// Check the status of the nominatim server.
    pub async fn status() -> Result<(), NominatimError> {
        let plaintext = match surf::get("https://nominatim.openstreetmap.org/status.php?format=json").recv_string().await {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Http(error.to_string())),
        };

        let json: Value = match serde_json::from_str(&plaintext) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        let status = match &json["status"] {
            Value::Number(s) => match s.as_u64() {
                Some(n) => n as usize,
                None => {
                    return Err(NominatimError::Json(
                        "couldn't find the geocoded place_id as a number".to_string(),
                    ))
                }
            },
            _ => return Err(NominatimError::Json("No Status Code".to_string())),
        };

        match status {
            0 => return Ok(()),
            700 => return Err(NominatimError::Http("No database".to_string())),
            701 => return Err(NominatimError::Http("Module failed".to_string())),
            702 => return Err(NominatimError::Http("Module call failed".to_string())),
            703 => return Err(NominatimError::Http("Query failed".to_string())),
            704 => return Err(NominatimError::Http("No value".to_string())),
            _ => return Err(NominatimError::Http(status.to_string())),
        }
    }

    fn parse(geocode_json: &Value) -> Result<Self, NominatimError> {
        let latitude = match &geocode_json["lat"] {
            Value::String(s) => s.clone(),
            _ => "UNKNOWN".to_string(),
        };

        let longitude = match &geocode_json["lon"] {
            Value::String(s) => s.clone(),
            _ => "UNKNOWN".to_string(),
        };

        let latitude: f64 = match latitude.parse() {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        let longitude: f64 = match longitude.parse() {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        let location = match &geocode_json["display_name"] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(NominatimError::Json(
                    "couldn't find the geocoded location".to_string(),
                ))
            }
        };

        let place_id = match &geocode_json["place_id"] {
            Value::Number(s) => match s.as_u64() {
                Some(n) => n as usize,
                None => {
                    return Err(NominatimError::Json(
                        "couldn't find the geocoded place_id as a number".to_string(),
                    ))
                }
            },
            _ => 0,
        };

        let osm_id = match &geocode_json["osm_id"] {
            Value::Number(s) => match s.as_u64() {
                Some(n) => n as usize,
                None => {
                    return Err(NominatimError::Json(
                        "couldn't find the geocoded osm_id as a number".to_string(),
                    ))
                }
            },
            _ => 0,
        };

        let address: Option<Address> = match geocode_json.get("address") {
            Some(data) => Address::from_json(data),
            None => None,
        };

        return Ok(Self {
            latitude,
            longitude,
            location: location.clone(),
            place_id: place_id.clone(),
            osm_id: osm_id.clone(),
            address,
        });
    }
}

/// An error enum for Nominatim.
#[derive(Error, Debug)]
pub enum NominatimError {
    #[error("http error {0}")]
    Http(String),
    #[error("json error {0}")]
    Json(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Address {
    pub house_number: Option<String>,
    pub road: Option<String>,
    pub suburb: Option<String>,
    pub neighbourhood: Option<String>,
    pub town: Option<String>,
    pub city: Option<String>,
    pub county: Option<String>,
    pub state: Option<String>,
    pub postcode: Option<usize>,
    pub country: Option<String>,
    pub country_code: Option<String>,
}

impl Address {
    pub fn from_json(data: &Value) -> Option<Self> {
        let house_number = match &data["house_number"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let road = match &data["road"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let suburb = match &data["suburb"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let neighbourhood = match &data["neighbourhood"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let town = match &data["town"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let city = match &data["city"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let county = match &data["county"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let state = match &data["state"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let postcode = match &data["postcode"] {
            Value::Number(s) => match s.as_u64() {
                Some(n) => Some(n as usize),
                None => None,
            },
            _ => None,
        };

        let country = match &data["country"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let country_code = match &data["country_code"] {
            Value::String(s) => Some(s.clone()),
            _ => None,
        };

        let address = Address {
            house_number,
            road,
            suburb,
            neighbourhood,
            town,
            city,
            county,
            state,
            postcode,
            country,
            country_code,
        };

        return Some(address);
    }
}