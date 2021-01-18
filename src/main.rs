#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

use {
    std::{
        fmt,
        io,
        process::exit,
    },
    derive_more::From,
    lazy_static::lazy_static,
    parking_lot::RwLock,
    regex::Regex,
    twitchchat::{
        PrivmsgExt as _,
        messages::Commands,
        runner::{
            AsyncRunner,
            Status,
        },
    },
    crate::{
        commands::COMMANDS,
        config::Config,
        data::Data,
        state::State,
    },
};

mod commands;
mod config;
mod data;
mod state;

lazy_static! {
    static ref COMMAND_REGEX: Regex = Regex::new("^!([a-z]+)(?: (.*))?$").expect("failed to build command regex");
}

#[derive(Debug, From)]
enum Error {
    Config(crate::config::Error),
    DataSave(crate::data::SaveError),
    Io(io::Error),
    Json(serde_json::Error),
    Runner(twitchchat::RunnerError),
    Shlex(shlex::Error),
    UserConfig(twitchchat::twitch::UserConfigError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(e) => write!(f, "config error: {}", e),
            Error::DataSave(e) => e.fmt(f),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::Runner(e) => write!(f, "runner error: {}", e),
            Error::Shlex(e) => write!(f, "error parsing command arguments: {}", e),
            Error::UserConfig(e) => write!(f, "error generating chat user config: {}", e),
        }
    }
}

async fn main_inner() -> Result<(), Error> {
    let config = Config::new()?;
    let mut data = Data::new()?;
    let state = RwLock::new(State::default());
    let user_config = config.user_config()?;
    let connector = twitchchat::connector::tokio::Connector::twitch()?;
    let mut runner = AsyncRunner::connect(connector, &user_config).await?;
    eprintln!("connecting, we are: {}", runner.identity.username());
    eprintln!("joining: #{}", config.channel_username);
    runner.join(&config.channel_username).await?;
    eprintln!("starting main loop");
    let mut writer = runner.writer();
    loop {
        match runner.next_message().await? {
            Status::Message(Commands::Privmsg(pm)) => {
                if let Some(captures) = COMMAND_REGEX.captures(pm.data()) {
                    if let Some(command) = COMMANDS.get(&captures[1]) {
                        command(pm, &mut writer, &state)?;
                    } else {
                        writer.say(&pm, "unknown command")?;
                        data.log_unknown_command(captures[1].to_owned(), pm);
                        data.save()?;
                    }
                }
            }
            Status::Message(_) => {}
            Status::Quit | Status::Eof => break, //TODO auto-reconnect?
            //TODO handle “stopped streaming” events to clear state
        }
    }
    eprintln!("end of main loop");
    Ok(())
}

#[tokio::main]
async fn main() {
    match main_inner().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("error in piuda-twitch: {}", e);
            exit(1)
        }
    }
}
