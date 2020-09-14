use {
    std::{
        collections::BTreeMap,
        fmt,
        fs::File,
        io
    },
    chrono::prelude::*,
    derive_more::From,
    serde::{
        Deserialize,
        Serialize
    },
    twitchchat::messages::Privmsg
};

const DATA_PATH: &str = "fenhl/piuda-twitch.json";

#[derive(Debug, From)]
pub(crate) enum SaveError {
    Basedir(xdg_basedir::Error),
    Io(io::Error),
    Json(serde_json::Error)
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Basedir(e) => e.fmt(f),
            SaveError::Io(e) => write!(f, "I/O error: {}", e),
            SaveError::Json(e) => write!(f, "error encoding JSON: {}", e)
        }
    }
}

#[derive(Deserialize, Serialize)]
struct UnknownCommand {
    text: String,
    timestamp: DateTime<Utc>
}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Data {
    #[serde(default)]
    unknown_commands: BTreeMap<String, Vec<UnknownCommand>>
}

impl Data {
    pub(crate) fn new() -> Result<Data, serde_json::Error> {
        let dirs = xdg_basedir::get_data_home().into_iter().chain(xdg_basedir::get_data_dirs());
        Ok(dirs.filter_map(|data_dir| File::open(data_dir.join(DATA_PATH)).ok())
            .next().map_or(Ok(Data::default()), serde_json::from_reader)?)
    }

    pub(crate) fn log_unknown_command(&mut self, name: String, pm: Privmsg<'_>) {
        self.unknown_commands
            .entry(name)
            .or_insert_with(Vec::default)
            .push(UnknownCommand { text: pm.data().to_owned(), timestamp: Utc::now() });
    }

    pub(crate) fn save(&self) -> Result<(), SaveError> {
        let dirs = xdg_basedir::get_data_home().into_iter().chain(xdg_basedir::get_data_dirs());
        for data_dir in dirs {
            let data_path = data_dir.join(DATA_PATH);
            if data_path.exists() {
                if let Some(()) = File::create(data_path).ok()
                    .and_then(|data_file| serde_json::to_writer_pretty(data_file, &self).ok())
                {
                    return Ok(());
                }
            }
        }
        let data_path = xdg_basedir::get_data_home()?.join(DATA_PATH);
        let data_file = File::create(data_path)?;
        serde_json::to_writer_pretty(data_file, &self)?;
        Ok(())
    }
}
