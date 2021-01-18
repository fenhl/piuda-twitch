use {
    std::collections::HashMap,
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

type Command = &'static (dyn Fn(Privmsg<'_>, &mut twitchchat::Writer, &RwLock<State>) -> Result<(), Error> + Sync);

macro_rules! commands {
    ($($name:ident$(, $alias:ident)*($privmsg:pat, $writer:pat, $state:pat) $block:block)*) => {
        $(
            fn $name($privmsg: Privmsg<'_>, $writer: &mut twitchchat::Writer, $state: &RwLock<State>) -> Result<(), Error> $block
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
    arena, id(privmsg, writer, state) {
        //TODO check whether I'm playing Smash Ultimate
        let args = shlex::split(privmsg.data())?;
        match args.first().map(|arg| &arg[..]) {
            Some("set") => if privmsg.is_broadcaster() || privmsg.is_moderator() {
                state.write().arena = args.get(1).cloned();
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

    ping(privmsg, writer, _) {
        writer.say(&privmsg, "pong")?;
        Ok(())
    }
}
