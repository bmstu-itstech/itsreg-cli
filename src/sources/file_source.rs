use std::path::PathBuf;

use crate::models::CreateScriptRequest;
use crate::sources::Source;

pub struct FileSource {
    path: PathBuf,
}

impl FileSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        FileSource { path: path.into() }
    }
}

impl Source for FileSource {
    fn input_script(&self) -> std::io::Result<CreateScriptRequest> {
        let s = std::fs::read_to_string(&self.path)?;
        serde_json::from_str(&s).map_err(Into::into)
    }
}
