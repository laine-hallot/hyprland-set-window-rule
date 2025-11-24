use clap::{Parser, Subcommand, ValueEnum};

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
#[derive(Clone, ValueEnum)]
pub enum SelectWindowBy {
    Title,
    Class,
    InitialClass,
    InitialTitle,
}

#[derive(Subcommand)]
pub enum Commands {
    Generate {
        #[arg(long, help = "add float rule", conflicts_with = "tile")]
        float: bool,

        #[arg(long, help = "add tile rule", conflicts_with = "float")]
        tile: bool,

        #[arg(long, help = "add fullscreen rule")]
        fullscreen: bool,

        #[arg(long, help = "name of value to use in the windowrule query")]
        select_by: Vec<SelectWindowBy>,
    },
}
