use serde::Deserialize;
use uuid::Uuid;

const SPACE_BLACKLIST_FILE: &str = "spaces_blacklist.yaml";

#[derive(Clone, Debug, Deserialize)]
pub struct SpacesBlacklist {
    pub spaces: Vec<Uuid>,
}

pub fn load() -> anyhow::Result<Option<SpacesBlacklist>> {
    if !std::path::Path::new(SPACE_BLACKLIST_FILE).exists() {
        return Ok(None);
    }

    let blacklist = std::fs::read_to_string(SPACE_BLACKLIST_FILE)?;
    let blacklist: SpacesBlacklist = serde_yaml::from_str(&blacklist)?;
    Ok(Some(blacklist))
}
