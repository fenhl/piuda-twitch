use {
    std::{
        fmt,
        fs::File
    },
    derive_more::From,
    serde::Deserialize,
    twitchchat::{
        UserConfig,
        twitch::UserConfigError
    }
};

#[derive(Debug, From)]
pub(crate) enum Error {
    Json(serde_json::Error),
    MissingConfig
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Json(e) => write!(f, "error decoding JSON: {}", e),
            Error::MissingConfig => write!(f, "missing config file")
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Config {
    #[serde(default = "make_piuda")]
    pub(crate) bot_username: String,
    #[serde(default = "make_fenhl")]
    pub(crate) channel_username: String,
    pub(crate) token: String
}

impl Config {
    pub(crate) fn new() -> Result<Config, Error> {
        let dirs = xdg_basedir::get_config_home().into_iter().chain(xdg_basedir::get_config_dirs());
        let file = dirs.filter_map(|cfg_dir| File::open(cfg_dir.join("fenhl/piuda-twitch.json")).ok())
            .next().ok_or(Error::MissingConfig)?;
        Ok(serde_json::from_reader(file)?)
    }

    pub(crate) fn user_config(&self) -> Result<UserConfig, UserConfigError> {
        UserConfig::builder()
            .name(&self.bot_username)
            .token(format!("oauth:{}", self.token))
            .enable_all_capabilities()
            .build()
    }
}

fn make_fenhl() -> String { format!("fenhl") }
fn make_piuda() -> String { format!("piuda") }
