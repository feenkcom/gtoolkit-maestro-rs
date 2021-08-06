mod builder;
mod checker;
mod cleaner;
mod copier;
mod downloader;
mod release;
mod setup;
mod starter;
mod tentative;
mod tester;

use console::Emoji;

pub use builder::{BuildOptions, Builder, Loader, ReleaseBuildOptions};
pub use checker::Checker;
pub use cleaner::Cleaner;
pub use copier::{Copier, CopyOptions};
pub use downloader::Downloader;
pub use release::{Release, ReleaseOptions};
pub use setup::{Setup, SetupOptions, SetupTarget};
pub use starter::{StartOptions, Starter};
pub use tentative::{Tentative, TentativeOptions};
pub use tester::{TestOptions, Tester};

pub static CHECKING: Emoji<'_, '_> = Emoji("🔍 ", "");
pub static DOWNLOADING: Emoji<'_, '_> = Emoji("📥 ", "");
pub static EXTRACTING: Emoji<'_, '_> = Emoji("📦 ", "");
pub static MOVING: Emoji<'_, '_> = Emoji("🚚 ", "");
pub static CREATING: Emoji<'_, '_> = Emoji("📝 ", "");
pub static BUILDING: Emoji<'_, '_> = Emoji("🔨 ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
