mod builder;
mod setup;
mod tester;

pub use builder::{BuildOptions, Builder};
use console::Emoji;
pub use setup::Setup;

pub static CHECKING: Emoji<'_, '_> = Emoji("🔍 ", "");
pub static DOWNLOADING: Emoji<'_, '_> = Emoji("📥 ", "");
pub static EXTRACTING: Emoji<'_, '_> = Emoji("📦 ", "");
pub static MOVING: Emoji<'_, '_> = Emoji("🚚 ", "");
pub static CREATING: Emoji<'_, '_> = Emoji("📝 ", "");
pub static BUILDING: Emoji<'_, '_> = Emoji("🏗️  ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
