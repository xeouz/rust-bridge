use std::fs;

use rocket::figment::value::Value;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    ip: String,
    port: Option<u16>,
    documents: Value
}

#[derive(Debug)]
pub struct FileReadError;

#[derive(Clone)]
pub struct HydratedConfig {
    ip: String,
    port: u16,
    documents: Vec<(String, String)>
}

impl HydratedConfig {
    pub fn new(ip: String, port: u16, documents: Vec<(String, String)>) -> Self {
        HydratedConfig { ip: ip, port: port, documents: documents }
    }

    pub fn get_ip(&self) -> &str { self.ip.as_str() }
    pub fn get_port(&self) -> u16 { self.port }
    pub fn get_documents(&self) -> &Vec<(String, String)> { &self.documents }
}

pub fn read_file(fname: &str) -> Result<String, FileReadError> {
    let contents  = fs::read_to_string(fname);

    if contents.is_err() {
        panic!("read_file: Could not read `{}`", fname)
    }

    Ok(contents.unwrap())
}

pub fn read_toml(fpath: &str) -> Result<HydratedConfig, FileReadError> {
    let stripped_fpath = if fpath.ends_with("/") { fpath.strip_suffix("/").unwrap() } else { fpath }.to_owned();
    let toml_path = stripped_fpath.to_string() + "/Athen.toml";
    let contents = read_file(&toml_path).unwrap();
    let data: Config = toml::from_str(contents.as_str()).unwrap();

    let ip = data.ip;
    let port = if data.port.is_none() { 8000 } else { data.port.unwrap() };
    let mut documents: Vec<(String, String)> = Vec::new();

    if data.documents.as_dict().is_none() {
        panic!()
    }

    for (k,v) in data.documents.as_dict().unwrap().iter() {
        match v {
            Value::String(_string, _) => (),
            _ => panic!()
        }

        let path = stripped_fpath.to_string() + "/" + v.as_str().unwrap();
        documents.push((k.to_string(), path));
    }

    Ok(HydratedConfig::new(ip, port, documents))
}