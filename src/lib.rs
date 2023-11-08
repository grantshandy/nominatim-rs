#![doc = include_str!("../README.md")]

use std::{str::FromStr, time::Duration};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use url::Url;

mod ident;

pub use ident::IdentificationMethod;

/// The interface for accessing a Nominatim API server.
#[derive(Debug, Clone)]
pub struct Client {
    ident: IdentificationMethod, // how to access the server
    base_url: Url,               // defaults to https://nominatim.openstreetmap.org
    client: reqwest::Client,
    /// HTTP Request Timeout [`Duration`]
    pub timeout: Duration,
}

impl Client {
    /// Create a new [`Client`] from an [`IdentificationMethod`].
    pub fn new(ident: IdentificationMethod) -> Self {
        let timeout = Duration::from_secs(10);

        Self {
            ident,
            base_url: Url::parse("https://nominatim.openstreetmap.org/").unwrap(),
            client: reqwest::ClientBuilder::new()
                .timeout(timeout)
                .build()
                .unwrap(),
            timeout,
        }
    }

    /// Set the client's internal base url for all requests.
    pub fn set_base_url<U: TryInto<Url>>(&mut self, url: U) -> Result<(), U::Error> {
        self.base_url = url.try_into()?;

        Ok(())
    }

    /// Check the status of the nominatim server.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Status/)
    pub async fn status(&self) -> Result<Status, reqwest::Error> {
        let mut url = self.base_url.join("status.php").unwrap();
        url.set_query(Some("format=json"));

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).expect("invalid nominatim auth header name"),
            HeaderValue::from_str(&self.ident.value())
                .expect("invalid nominatim auth header value"),
        );

        self.client
            .get(url)
            .headers(headers)
            .timeout(self.timeout)
            .send()
            .await?
            .json()
            .await
    }

    /// Get [`Place`]s from a search query.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Search/)
    pub async fn search(&self, query: impl AsRef<str>) -> Result<Vec<Place>, reqwest::Error> {
        let mut url = self.base_url.clone();
        url.set_query(Some(&format!(
            "addressdetails=1&extratags=1&q={}&format=json",
            query.as_ref().replace(' ', "+")
        )));

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).expect("invalid nominatim auth header name"),
            HeaderValue::from_str(&self.ident.value())
                .expect("invalid nominatim auth header value"),
        );

        self.client
            .get(url)
            .headers(headers)
            .timeout(self.timeout)
            .send()
            .await?
            .json()
            .await
    }

    /// Generate a [`Place`] from latitude and longitude.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Reverse/)
    pub async fn reverse(
        &self,
        latitude: impl AsRef<str>,
        longitude: impl AsRef<str>,
        zoom: Option<u8>,
    ) -> Result<Place, reqwest::Error> {
        let mut url = self.base_url.join("reverse").unwrap();

        match zoom {
            Some(zoom) => {
                url.set_query(Some(&format!(
                    "addressdetails=1&extratags=1&format=json&lat={}&lon={}&zoom={}",
                    latitude.as_ref().replace(' ', ""),
                    longitude.as_ref().replace(' ', ""),
                    zoom
                )));
            }
            None => {
                url.set_query(Some(&format!(
                    "addressdetails=1&extratags=1&format=json&lat={}&lon={}",
                    latitude.as_ref().replace(' ', ""),
                    longitude.as_ref().replace(' ', ""),
                )));
            }
        }

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).expect("invalid nominatim auth header name"),
            HeaderValue::from_str(&self.ident.value())
                .expect("invalid nominatim auth header value"),
        );

        self.client
            .get(url)
            .headers(headers)
            .timeout(self.timeout)
            .send()
            .await?
            .json()
            .await
    }

    /// Return [`Place`]s from a list of OSM Node, Way, or Relations.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Lookup/)
    pub async fn lookup(&self, queries: Vec<&str>) -> Result<Vec<Place>, reqwest::Error> {
        let queries = queries.join(",");

        let mut url = self.base_url.join("lookup").unwrap();
        url.set_query(Some(&format!(
            "osm_ids={}&addressdetails=1&extratags=1&format=json",
            queries
        )));

        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str(&self.ident.header()).expect("invalid nominatim auth header name"),
            HeaderValue::from_str(&self.ident.value())
                .expect("invalid nominatim auth header value"),
        );

        self.client
            .get(url)
            .headers(headers)
            .timeout(self.timeout)
            .send()
            .await?
            .json()
            .await
    }
}

/// The status of a Nominatim server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Status {
    pub status: usize,
    pub message: String,
    pub data_updated: Option<String>,
    pub software_version: Option<String>,
    pub database_version: Option<String>,
}

/// A location returned by the Nominatim server.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Place {
    #[serde(default)]
    pub place_id: usize,
    #[serde(default)]
    pub licence: String,
    #[serde(default)]
    pub osm_type: String,
    #[serde(default)]
    pub osm_id: usize,
    #[serde(default)]
    pub boundingbox: Vec<String>,
    #[serde(default)]
    pub lat: String,
    #[serde(default)]
    pub lon: String,
    #[serde(default)]
    pub display_name: String,
    pub class: Option<String>,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub importance: Option<f64>,
    pub icon: Option<String>,
    #[serde(default)]
    pub address: Option<Address>,
    pub extratags: Option<ExtraTags>,
}

/// An address for a place.
///
/// [Address details](https://nominatim.org/release-docs/latest/api/Output/#addressdetails)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Address {
    pub continent: Option<String>,

    pub country: Option<String>,
    pub country_code: Option<String>,

    pub region: Option<String>,
    pub state: Option<String>,
    pub state_district: Option<String>,
    pub county: Option<String>,
    #[serde(rename = "ISO3166-2-lvl4")]
    pub iso3166_2_lvl4: Option<String>,

    pub municipality: Option<String>,
    pub city: Option<String>,
    pub town: Option<String>,
    pub village: Option<String>,

    pub city_district: Option<String>,
    pub district: Option<String>,
    pub borough: Option<String>,
    pub suburb: Option<String>,
    pub subdivision: Option<String>,

    pub hamlet: Option<String>,
    pub croft: Option<String>,
    pub isolated_dwelling: Option<String>,

    pub neighbourhood: Option<String>,
    pub allotments: Option<String>,
    pub quarter: Option<String>,

    pub city_block: Option<String>,
    pub residential: Option<String>,
    pub farm: Option<String>,
    pub farmyard: Option<String>,
    pub industrial: Option<String>,
    pub commercial: Option<String>,
    pub retail: Option<String>,

    pub road: Option<String>,

    pub house_number: Option<String>,
    pub house_name: Option<String>,

    pub emergency: Option<String>,
    pub historic: Option<String>,
    pub military: Option<String>,
    pub natural: Option<String>,
    pub landuse: Option<String>,
    pub place: Option<String>,
    pub railway: Option<String>,
    pub man_made: Option<String>,
    pub aerialway: Option<String>,
    pub boundary: Option<String>,
    pub amenity: Option<String>,
    pub aeroway: Option<String>,
    pub club: Option<String>,
    pub craft: Option<String>,
    pub leisure: Option<String>,
    pub office: Option<String>,
    pub mountain_pass: Option<String>,
    pub shop: Option<String>,
    pub tourism: Option<String>,
    pub bridge: Option<String>,
    pub tunnel: Option<String>,
    pub waterway: Option<String>,

    pub postcode: Option<String>,
}

/// Extra metadata that a place may have.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraTags {
    pub capital: Option<String>,
    pub website: Option<String>,
    pub wikidata: Option<String>,
    pub wikipedia: Option<String>,
    pub population: Option<String>,
}
