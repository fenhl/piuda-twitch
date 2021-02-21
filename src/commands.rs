use {
    std::collections::{
        HashMap,
        HashSet,
    },
    lazy_static::lazy_static,
    parking_lot::RwLock,
    twitchchat::{
        PrivmsgExt as _,
        messages::Privmsg,
    },
    crate::{
        Error,
        state::State,
    },
};

type Command = &'static (dyn Fn(Privmsg<'_>, &mut twitchchat::Writer, &RwLock<State>, &HashSet<&'static str>) -> Result<(), Error> + Sync);

macro_rules! commands {
    ($($name:ident$(, $alias:ident)*($privmsg:pat, $writer:pat, $state:pat, $commands:pat) $block:block)*) => {
        $(
            fn $name($privmsg: Privmsg<'_>, $writer: &mut twitchchat::Writer, $state: &RwLock<State>, $commands: &HashSet<&'static str>) -> Result<(), Error> $block
        )*

        lazy_static! {
            pub(crate) static ref COMMANDS: HashMap<&'static str, Command> = {
                let mut commands = HashMap::<&'static str, Command>::default();
                $(
                    commands.insert(stringify!($name), &$name);
                    $(
                        commands.insert(stringify!($alias), &$name);
                    )*
                )*
                commands
            };
        }
    };
}

commands! {
    arena, id(privmsg, writer, state, _) {
        //TODO check whether I'm playing Smash Ultimate
        let args = shlex::split(privmsg.data()).ok_or(Error::Shlex)?;
        match args.first().map(|arg| &arg[..]) {
            Some("set") => if privmsg.is_broadcaster() || privmsg.is_moderator() {
                state.write().arena = args.get(1).cloned(); //TODO validate Smash Ultimate arena ID
                writer.say(&privmsg, "arena ID updated")?;
            } else {
                writer.say(&privmsg, "this subcommand is moderator-only")?;
            },
            //TODO other subcommands
            Some(_) => writer.say(&privmsg, "unknown !arena subcommand")?,
            None => if let Some(ref arena_id) = state.read().arena {
                writer.say(&privmsg, &format!("arena ID: {} • password: 4813", arena_id))?;
            } else {
                //writer.say(&privmsg, "sorry, but Fenhl is not playing with viewers right now")?, //TODO if “playing with viewers” tag is absent
                writer.say(&privmsg, "no arena running, Fenhl will open one after this game/tourney")?;
            },
        }
        Ok(())
    }

    command, cmd(privmsg, writer, state, commands) {
        if !privmsg.is_broadcaster() && !privmsg.is_moderator() {
            writer.say(&privmsg, "this subcommand is moderator-only")?;
            return Ok(())
        }
        let args = shlex::split(privmsg.data()).ok_or(Error::Shlex)?;
        match args.first().map(|arg| &arg[..]) {
            Some("add") | Some("create") | Some("new") => {
                if let [cmd_name, cmd_text] = &args[2..] {
                    if commands.contains(&(&cmd_name[..])) {
                        writer.say(&privmsg, "this is a built-in command that can't be overwritten")?;
                        return Ok(())
                    }
                    let mut state = state.write();
                    if state.simple_commands.contains_key(cmd_name) {
                        writer.say(&privmsg, "command already exists, use `cmd edit` to overwrite it")?;
                    } else {
                        state.simple_commands.insert(cmd_name.to_owned(), cmd_text.to_owned());
                        writer.say(&privmsg, "command added")?;
                    }
                } else {
                    writer.say(&privmsg, "`cmd add` takes exactly 2 arguments: command name and command text")?;
                }
            }
            Some("edit") => {
                if let [cmd_name, cmd_text] = &args[2..] {
                    if commands.contains(&(&cmd_name[..])) {
                        writer.say(&privmsg, "this is a built-in command that can't be overwritten")?;
                        return Ok(())
                    }
                    let mut state = state.write();
                    if state.simple_commands.contains_key(cmd_name) {
                        state.simple_commands.insert(cmd_name.to_owned(), cmd_text.to_owned());
                        writer.say(&privmsg, "command edited")?;
                    } else {
                        writer.say(&privmsg, "command does not exist, use `cmd add` to create it")?;
                    }
                } else {
                    writer.say(&privmsg, "`cmd edit` takes exactly 2 arguments: command name and new command text")?;
                }
            }
            Some("del") | Some("delete") | Some("rm") => {
                if let [cmd_name] = &args[2..] {
                    if state.write().simple_commands.remove(cmd_name).is_some() {
                        writer.say(&privmsg, "command deleted")?;
                    } else {
                        writer.say(&privmsg, "command does not exist")?;
                    }
                } else {
                    writer.say(&privmsg, "`cmd del` takes exactly 1 argument")?;
                }
            }
            Some(_) => writer.say(&privmsg, "unknown !cmd subcommand")?,
            None => writer.say(&privmsg, "missing !cmd subcommand")?,
        }
        Ok(())
    }

    ping(privmsg, writer, _, _) {
        writer.say(&privmsg, "pong")?;
        Ok(())
    }
}
