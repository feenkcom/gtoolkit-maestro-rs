mod builder;
mod checker;
mod downloader;
mod setup;
mod tentative;
mod tester;

use console::Emoji;

pub use builder::{BuildOptions, Builder, Loader};
pub use checker::Checker;
pub use downloader::Downloader;
pub use setup::{Setup, SetupOptions, SetupTarget};
pub use tentative::{Tentative, TentativeOptions};
pub use tester::Tester;

pub static CHECKING: Emoji<'_, '_> = Emoji("🔍 ", "");
pub static DOWNLOADING: Emoji<'_, '_> = Emoji("📥 ", "");
pub static EXTRACTING: Emoji<'_, '_> = Emoji("📦 ", "");
pub static MOVING: Emoji<'_, '_> = Emoji("🚚 ", "");
pub static CREATING: Emoji<'_, '_> = Emoji("📝 ", "");
pub static BUILDING: Emoji<'_, '_> = Emoji("🏗️ ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
