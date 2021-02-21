use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct State {
    pub(crate) arena: Option<String>,
    pub(crate) simple_commands: HashMap<String, String>,
}
