use clap::{Args, Parser, Subcommand};

use crate::NeoError;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "neo")]
pub enum Cli {
    Add(AddArgs),
    Pop(PopArgs),
    Delete(DeleteArgs),
    Size,
    Frontier(FrontierArgs),
    Directory,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct AddArgs {
    pub url: String,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct PopArgs {
    pub index: Option<usize>,
    #[arg(long)]
    pub open: bool,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct DeleteArgs {
    pub url: String,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct FrontierArgs {
    #[command(subcommand)]
    pub command: FrontierCommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum FrontierCommand {
    Start(NameArgs),
    Switch(NameArgs),
    Rename(RenameArgs),
    List,
    Delete(NameArgs),
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct NameArgs {
    pub name: String,
}

#[derive(Args, Debug, PartialEq, Eq)]
pub struct RenameArgs {
    pub name: String,
    pub new_name: String,
}

impl Cli {
    pub fn parse<I>(args: I) -> Result<Self, NeoError>
    where
        I: IntoIterator<Item = String>,
    {
        let args = std::iter::once(String::from("neo")).chain(args);
        Self::try_parse_from(args).map_err(|err| NeoError::Usage(err.to_string()))
    }
}
