use serde_json::Value;
use thiserror::Error;

pub enum IdentificationMethod {
    Referer(String),
    UserAgent(String),
}

impl IdentificationMethod {
    pub fn header(&self) -> String {
        match self {
            Self::Referer(_) => "Referer".to_string(),
            Self::UserAgent(_) => "User-Agent".to_string(),
        }
    }
    pub fn value(&self) -> String {
        match self {
            Self::Referer(value) => value.to_string(),
            Self::UserAgent(value) => value.to_string(),
        }
    }
}

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
pub struct NominatimClient {
    pub identification: IdentificationMethod,
}

impl NominatimClient {
    /// Get data from an openstreetmap ID.
    pub async fn lookup<T: AsRef<str>>(self, osm_id: T) -> Result<Nominatim, NominatimError> {
        let uri = &format!(
            "https://nominatim.openstreetmap.org/lookup?osm_ids={}&format=json",
            osm_id.as_ref().replace(" ", "")
        );

        let geocode = self.get(uri).await?;

        let geocode_json: Value = match serde_json::from_str(&geocode) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        Nominatim::parse(&geocode_json[0])
    }

    pub async fn get<T: AsRef<str>>(self, uri: T) -> Result<String, NominatimError> {
        surf::get(uri)
            .header(
                self.identification.header().as_str(),
                self.identification.value().as_str(),
            )
            .recv_string()
            .await
            .map_err(|error| NominatimError::Http(error.to_string()))
    }

    /// Get data from the name of a location.
    pub async fn search<T: AsRef<str>>(self, name: T) -> Result<Nominatim, NominatimError> {
        let uri = &format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json",
            name.as_ref().replace(" ", "+")
        );

        let geocode = self.get(uri).await?;

        let geocode_json: Value = match serde_json::from_str(&geocode) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        Nominatim::parse(&geocode_json[0])
    }

    /// Get data from the coordinates of a location.
    pub async fn reverse(self, lat: f64, lon: f64) -> Result<Nominatim, NominatimError> {
        let uri = &format!(
            "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json",
            lat, lon
        );
        let geocode = self.get(uri).await?;

        let geocode_json: Value = match serde_json::from_str(&geocode) {
            Ok(data) => data,
            Err(error) => return Err(NominatimError::Json(error.to_string())),
        };

        Nominatim::parse(&geocode_json)
    }

    /// Check the status of the nominatim server.
    pub async fn status(self) -> Result<(), NominatimError> {
        let plaintext = self
            .get("https://nominatim.openstreetmap.org/status.php?format=json")
            .await?;

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
            0 => Ok(()),
            700 => Err(NominatimError::Http("No database".to_string())),
            701 => Err(NominatimError::Http("Module failed".to_string())),
            702 => Err(NominatimError::Http("Module call failed".to_string())),
            703 => Err(NominatimError::Http("Query failed".to_string())),
            704 => Err(NominatimError::Http("No value".to_string())),
            _ => Err(NominatimError::Http(status.to_string())),
        }
    }
}

impl Nominatim {
    pub fn parse(geocode_json: &Value) -> Result<Self, NominatimError> {
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

        Ok(Self {
            latitude,
            longitude,
            location,
            place_id,
            osm_id,
            address,
        })
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

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Address {
    pub house_number: Option<String>,
    pub road: Option<String>,
    pub village: Option<String>,
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

        let village = match &data["village"] {
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
            Value::Number(s) => s.as_u64().map(|n| n as usize),
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
            village,
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

        Some(address)
    }
}
