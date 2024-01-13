/// Which method to access the nominatim API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentificationMethod {
    Referer(String),
    UserAgent(String),
}

impl IdentificationMethod {
    pub fn header(&self) -> &'static str {
        match self {
            Self::Referer(_) => "Referer",
            Self::UserAgent(_) => "User-Agent",
        }
    }

    pub fn value(self) -> String {
        match self {
            Self::Referer(value) => value,
            Self::UserAgent(value) => value,
        }
    }

    pub fn from_referer(s: impl AsRef<str>) -> Self {
        Self::Referer(s.as_ref().to_string())
    }

    pub fn from_user_agent(s: impl AsRef<str>) -> Self {
        Self::UserAgent(s.as_ref().to_string())
    }
}
