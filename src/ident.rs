/// Which method to access the nominatim API.
#[derive(Clone, Debug, PartialEq, Eq)]
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

    pub fn from_referer(s: impl AsRef<str>) -> Self {
        Self::Referer(s.as_ref().to_string())
    }

    pub fn from_user_agent(s: impl AsRef<str>) -> Self {
        Self::UserAgent(s.as_ref().to_string())
    }
}
