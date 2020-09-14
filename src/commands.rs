use {
    std::collections::HashMap,
    lazy_static::lazy_static,
    twitchchat::{
        PrivmsgExt as _,
        messages::Privmsg
    },
    crate::Error
};

type Command = &'static (dyn Fn(Privmsg<'_>, &mut twitchchat::Writer) -> Result<(), Error> + Sync);

lazy_static! {
    pub(crate) static ref COMMANDS: HashMap<&'static str, Command> = {
        let mut commands = HashMap::<&'static str, Command>::default();
        commands.insert("ping", &ping);
        commands
    };
}

fn ping(privmsg: Privmsg<'_>, writer: &mut twitchchat::Writer) -> Result<(), Error> {
    writer.say(&privmsg, "pong")?;
    Ok(())
}
