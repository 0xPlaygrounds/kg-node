use std::fmt::Display;

use crate::ids::Grc20Id;

pub struct GraphUri {
    pub id: String,
}

impl Display for GraphUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "graph://{}", self.id)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid graph uri: {0}")]
pub struct InvalidGraphUri(String);

impl GraphUri {
    pub fn from_id_str(id: &str) -> Self {
        Self { id: id.to_string() }
    }

    pub fn from_id(id: Grc20Id) -> Self {
        Self { id: id.to_string() }
    }

    pub fn to_id(&self) -> Grc20Id {
        Grc20Id(self.id.clone())
    }

    pub fn from_uri(uri: &str) -> Result<Self, InvalidGraphUri> {
        if !uri.starts_with("graph://") {
            Err(InvalidGraphUri(uri.to_string()))
        } else {
            Ok(Self {
                id: uri.replace("graph://", ""),
            })
        }
    }

    pub fn is_valid(uri: &str) -> bool {
        uri.starts_with("graph://")
    }
}
