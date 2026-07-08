use crate::models::CreateScriptRequest;

pub mod file_source;

pub trait Source {
    fn input_script(&self) -> std::io::Result<CreateScriptRequest>;
}
