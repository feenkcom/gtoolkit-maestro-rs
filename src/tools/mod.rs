mod builder;
mod packager;
mod setup;

use console::Emoji;

pub use builder::{BuildOptions, Builder, Loader};
pub use packager::Packager;
pub use setup::{Setup, SetupOptions, SetupTarget};

pub static CHECKING: Emoji<'_, '_> = Emoji("🔍 ", "");
pub static DOWNLOADING: Emoji<'_, '_> = Emoji("📥 ", "");
pub static EXTRACTING: Emoji<'_, '_> = Emoji("📦 ", "");
pub static MOVING: Emoji<'_, '_> = Emoji("🚚 ", "");
pub static CREATING: Emoji<'_, '_> = Emoji("📝 ", "");
pub static BUILDING: Emoji<'_, '_> = Emoji("🏗️ ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
