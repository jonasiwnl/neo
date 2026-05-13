use crate::NeoError;

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Add { url: String },
    Pop { open: bool },
    Delete { url: String },
    Frontier(FrontierCommand),
    Directory,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FrontierCommand {
    Start { name: String },
    Switch { name: String },
    Rename { name: String, new_name: String },
    List,
    Delete { name: String },
}

impl Cli {
    pub fn parse<I>(args: I) -> Result<Self, NeoError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let Some(command) = args.next() else {
            return Err(NeoError::usage());
        };

        match command.as_str() {
            "add" => Ok(Self::Add {
                url: take_one(args, "neo add <url>")?,
            }),
            "pop" => {
                let rest: Vec<_> = args.collect();
                match rest.as_slice() {
                    [] => Ok(Self::Pop { open: false }),
                    [flag] if flag == "--open" => Ok(Self::Pop { open: true }),
                    _ => Err(NeoError::Usage("usage: neo pop [--open]".into())),
                }
            }
            "delete" => Ok(Self::Delete {
                url: take_one(args, "neo delete <url>")?,
            }),
            "frontier" => Ok(Self::Frontier(parse_frontier(args)?)),
            "directory" => {
                let rest: Vec<_> = args.collect();
                if rest.is_empty() {
                    Ok(Self::Directory)
                } else {
                    Err(NeoError::Usage("usage: neo directory".into()))
                }
            }
            _ => Err(NeoError::usage()),
        }
    }
}

fn parse_frontier<I>(args: I) -> Result<FrontierCommand, NeoError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Err(NeoError::Usage(
            "usage: neo frontier <start|switch|rename|list|delete> ...".into(),
        ));
    };

    match command.as_str() {
        "start" => Ok(FrontierCommand::Start {
            name: take_one(args, "neo frontier start <name>")?,
        }),
        "switch" => Ok(FrontierCommand::Switch {
            name: take_one(args, "neo frontier switch <name>")?,
        }),
        "rename" => {
            let rest: Vec<_> = args.collect();
            match rest.as_slice() {
                [name, new_name] => Ok(FrontierCommand::Rename {
                    name: name.clone(),
                    new_name: new_name.clone(),
                }),
                _ => Err(NeoError::Usage(
                    "usage: neo frontier rename <name> <new_name>".into(),
                )),
            }
        }
        "list" => {
            let rest: Vec<_> = args.collect();
            if rest.is_empty() {
                Ok(FrontierCommand::List)
            } else {
                Err(NeoError::Usage("usage: neo frontier list".into()))
            }
        }
        "delete" => Ok(FrontierCommand::Delete {
            name: take_one(args, "neo frontier delete <name>")?,
        }),
        _ => Err(NeoError::Usage(
            "usage: neo frontier <start|switch|rename|list|delete> ...".into(),
        )),
    }
}

fn take_one<I>(args: I, usage: &str) -> Result<String, NeoError>
where
    I: IntoIterator<Item = String>,
{
    let values: Vec<_> = args.into_iter().collect();
    match values.as_slice() {
        [value] => Ok(value.clone()),
        _ => Err(NeoError::Usage(format!("usage: {usage}"))),
    }
}
