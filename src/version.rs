use feenk_releaser::Version;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppVersion(Version);

impl From<Version> for AppVersion {
    fn from(version: Version) -> Self {
        AppVersion(version)
    }
}

impl Deref for AppVersion {
    type Target = Version;
    fn deref(&self) -> &Version {
        &self.0
    }
}

impl Display for AppVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageVersion(Version);

impl From<Version> for ImageVersion {
    fn from(version: Version) -> Self {
        ImageVersion(version)
    }
}

impl Deref for ImageVersion {
    type Target = Version;
    fn deref(&self) -> &Version {
        &self.0
    }
}

impl Display for ImageVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
