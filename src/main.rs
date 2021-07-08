extern crate clap;
extern crate console;
extern crate octocrab as github;
extern crate regex;
extern crate reqwest;
extern crate semver;
extern crate serde;
extern crate serde_derive;
extern crate tokio;
extern crate tokio_stream;
extern crate tokio_util;
extern crate zip;

mod builder;
mod create;
mod download;
mod error;
mod moving;
mod options;
mod smalltalking;
mod unzip;

use crate::builder::Builder;
use crate::options::SubCommand;
use clap::Clap;
use options::BuildOptions;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let options: BuildOptions = BuildOptions::parse();

    match options.command() {
        SubCommand::Build(_) => {
            Builder::new().build(&options).await?;
        }
        SubCommand::Get => {}
        SubCommand::Clone => {}
    };

    Ok(())
}
