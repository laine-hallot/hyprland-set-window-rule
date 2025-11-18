use clap::{Parser, Subcommand};
use regex::Regex;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::num::ParseIntError;

#[derive(Parser)]
#[command(about, long_about = None)]
pub struct Cli {
    /// Optional name to operate on
    pub name: Option<String>,

    /// Display version info
    #[arg(short, long)]
    pub version: bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Generate {
        #[arg(long, help = "add float rule")]
        float: bool,

        #[arg(long, help = "add persistentsize rule")]
        persistentsize: bool,

        #[arg(long, help = "add tile rule")]
        tile: bool,

        #[arg(long, help = "add fullscreen rule")]
        fullscreen: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRegion {
    pub at: (i16, i16),
    pub size: (i16, i16),
}

impl Display for ClientRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{},{} {}x{}",
            self.at.0, self.at.1, self.size.0, self.size.1
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseRegionError {
    /// The string does not follow the expected “x,y wxh” layout.
    InvalidFormat,
    /// One of the integer components could not be parsed.
    InvalidNumber(ParseIntError),
}

impl std::fmt::Display for ParseRegionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ParseRegionError::InvalidFormat => write!(f, "invalid region format"),
            ParseRegionError::InvalidNumber(e) => write!(f, "invalid number: {}", e),
        }
    }
}

impl std::error::Error for ParseRegionError {}

impl From<ParseIntError> for ParseRegionError {
    fn from(err: ParseIntError) -> Self {
        ParseRegionError::InvalidNumber(err)
    }
}

impl TryFrom<&str> for ClientRegion {
    type Error = ParseRegionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let dimensions_regex = Regex::new(r"^\s*(-?\d+),\s*(-?\d+)\s+(-?\d+)x(-?\d+)\s*$")
            .expect("Could not create regex");

        let caps = dimensions_regex
            .captures(value)
            .ok_or(ParseRegionError::InvalidFormat)?;

        let x = caps[1].parse::<i16>()?;
        let y = caps[2].parse::<i16>()?;
        let w = caps[3].parse::<i16>()?;
        let h = caps[4].parse::<i16>()?;

        Ok(ClientRegion {
            at: (x, y),
            size: (w, h),
        })
    }
}
