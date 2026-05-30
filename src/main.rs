mod cli;
mod frontier;
mod index;
mod crawler;

#[cfg(test)]
mod tests;

use std::env;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;
use std::process::Command;

use cli::{Cli, FrontierCommand};
use frontier::FrontierRepo;

fn main() {
    if let Err(err) = run(env::args().skip(1), &mut io::stdout()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run<I>(args: I, stdout: &mut impl io::Write) -> Result<(), NeoError>
where
    I: IntoIterator<Item = String>,
{
    run_with_root(args, stdout, default_root_dir()?)
}

fn run_with_root<I>(args: I, stdout: &mut impl io::Write, root: PathBuf) -> Result<(), NeoError>
where
    I: IntoIterator<Item = String>,
{
    let command = Cli::parse(args)?;
    let repo = FrontierRepo::open(root)?;

    match command {
        Cli::Add(args) => repo.add_url(&args.url)?,
        Cli::Pop(args) => {
            let url = repo.pop_url(args.index)?;
            writeln!(stdout, "{url}")?;
            if args.open {
                open_in_browser(&url)?;
            }
        }
        Cli::Delete(args) => repo.delete_url(&args.url)?,
        Cli::Size => {
            writeln!(stdout, "{}", repo.size()?)?;
        }
        Cli::Index(args) => repo.index_command(args.library)?,
        Cli::Search(args) => {
            let urls = repo.search_command(&args.query)?;
            for url in &urls {
                writeln!(stdout, "{}", url)?;
            }
            
        }
        Cli::Frontier(frontier) => match frontier.command {
            FrontierCommand::Start(args) => repo.create_frontier(&args.name, true)?,
            FrontierCommand::Switch(args) => repo.switch_frontier(&args.name)?,
            FrontierCommand::Rename(args) => repo.rename_frontier(&args.name, &args.new_name)?,
            FrontierCommand::List => {
                let current = repo.current_frontier()?;
                for name in repo.list_frontiers()? {
                    let marker = if current.as_deref() == Some(name.as_str()) {
                        "*"
                    } else {
                        " "
                    };
                    writeln!(stdout, "{marker} {name}")?;
                }
            }
            FrontierCommand::Delete(args) => repo.delete_frontier(&args.name)?,
        },
        Cli::Directory => {
            writeln!(stdout, "{}", repo.root().display())?;
        }
    }

    Ok(())
}

pub fn default_root_dir() -> Result<PathBuf, NeoError> {
    if let Some(path) = env::var_os("NEO_ROOT_DIR") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(path).join("neo"));
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| NeoError::Message("HOME is not set".into()))?;
    Ok(home.join(".local").join("share").join("neo"))
}

fn open_in_browser(url: &str) -> Result<(), NeoError> {
    let status = Command::new("open").arg(url).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(NeoError::Message(format!(
            "failed to open '{url}' in browser"
        )))
    }
}

#[derive(Debug)]
pub enum NeoError {
    Io(io::Error),
    Reqwest(reqwest::Error),
    Usage(String),
    Message(String),
}

impl Display for NeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Reqwest(err) => write!(f, "{err}"),
            Self::Usage(message) | Self::Message(message) => f.write_str(message),
        }
    }
}

impl From<io::Error> for NeoError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for NeoError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
