#![doc = include_str!("../README.md")]

use std::{str::FromStr, time::Duration};

#[cfg(feature = "reqwest")]
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
#[cfg(feature = "wasm")]
use gloo::net::{self, http::{Request, Headers}};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use url::Url;

mod ident;

pub use ident::IdentificationMethod;

#[cfg(all(feature = "reqwest", feature = "wasm"))]
compile_error!("Features \"reqwest\" and \"wasm\" are mutually exclusive");

#[cfg(feature = "reqwest")]
type HttpClient = reqwest::Client;
#[cfg(feature = "wasm")]
type HttpClient = ();

#[cfg(feature = "reqwest")]
type Error = reqwest::Error;
#[cfg(feature = "wasm")]
type Error = net::Error;


/// The interface for accessing a Nominatim API server.
#[derive(Debug, Clone)]
pub struct Client {
    ident: IdentificationMethod, // how to access the server
    base_url: Url,               // defaults to https://nominatim.openstreetmap.org
    client: HttpClient,
    /// HTTP Request Timeout [`Duration`]
    pub timeout: Duration,
}

impl Client {
    /// Create a new [`Client`] from an [`IdentificationMethod`].
    pub fn new(ident: IdentificationMethod) -> Self {
        let timeout = Duration::from_secs(10);

        #[cfg(feature = "reqwest")]
        let client = reqwest::ClientBuilder::new()
            .timeout(timeout)
            .build()
            .unwrap();
        #[cfg(feature = "wasm")]
        let client = ();

        Self {
            ident,
            base_url: Url::parse("https://nominatim.openstreetmap.org/").unwrap(),
            client,
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
    pub async fn status(&self) -> Result<Status, Error> {
        let mut url = self.base_url.join("status.php").unwrap();
        url.set_query(Some("format=json"));

        let headers = mk_headers(&self.ident);

        fetch(&self.client, url, self.timeout, headers).await
    }

    /// Get [`Place`]s from a search query.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Search/)
    pub async fn search(&self, query: impl AsRef<str>) -> Result<Vec<Place>, Error> {
        let mut url = self.base_url.clone();
        url.set_query(Some(&format!(
            "addressdetails=1&extratags=1&q={}&format=json",
            query.as_ref().replace(' ', "+")
        )));

        let headers = mk_headers(&self.ident);

        fetch(&self.client, url, self.timeout, headers).await
    }

    /// Generate a [`Place`] from latitude and longitude.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Reverse/)
    pub async fn reverse(
        &self,
        latitude: impl AsRef<str>,
        longitude: impl AsRef<str>,
        zoom: Option<u8>,
    ) -> Result<Place, Error> {
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

        let headers = mk_headers(&self.ident);

        fetch(&self.client, url, self.timeout, headers).await
    }

    /// Return [`Place`]s from a list of OSM Node, Way, or Relations.
    ///
    /// [Documentation](https://nominatim.org/release-docs/develop/api/Lookup/)
    pub async fn lookup(&self, queries: Vec<&str>) -> Result<Vec<Place>, Error> {
        let queries = queries.join(",");

        let mut url = self.base_url.join("lookup").unwrap();
        url.set_query(Some(&format!(
            "osm_ids={}&addressdetails=1&extratags=1&format=json",
            queries
        )));

        let headers = mk_headers(&self.ident);

        fetch(&self.client, url, self.timeout, headers).await
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Address {
    pub city: Option<String>,
    pub state_district: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "ISO3166-2-lvl4")]
    pub iso3166_2_lvl4: Option<String>,
    pub postcode: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
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

#[cfg(feature = "reqwest")]
fn mk_headers(ident: &IdentificationMethod) -> HeaderMap {
    let mut hs = HeaderMap::new();
    hs.append(
        HeaderName::from_str(&ident.header())
            .expect("invalid nominatim auth header name"),
        HeaderValue::from_str(&ident.value())
            .expect("invalid nominatim auth header value"),
    );
    hs
}
#[cfg(feature = "wasm")]
fn mk_headers(ident: &IdentificationMethod) -> Headers {
    let hs = Headers::new();
    hs.append(
        &ident.header(),
        &ident.value(),
    );
    hs
}


#[cfg(feature = "reqwest")]
async fn fetch<T>(
    client: &HttpClient,
    url: Url,
    timeout: Duration,
    headers: HeaderMap
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    client
        .get(url)
        .headers(headers)
        .timeout(timeout)
        .send()
        .await?
        .json()
        .await
}

#[cfg(feature = "wasm")]
async fn fetch<T>(
    _client: &HttpClient,
    url: Url,
    _timeout: Duration,
    headers: Headers
) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    Request::get(url.as_str())
        .headers(headers)
    // .timeout(self.timeout)
        .send()
        .await?
        .json()
        .await
}
