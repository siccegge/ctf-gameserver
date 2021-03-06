use crate::Error;
use std::io::Write;
use std::io::BufRead;
use std::io::BufReader;

use std::fs::File;
use std::os::unix::io::FromRawFd;

use serde::{Deserialize, Serialize, de::DeserializeOwned};


pub trait ControlInterface {
    fn setup () -> Result<(), Error>;
    fn get_flag (&mut self, tick:u32, payload:&Vec<u8>) -> Result<String, Error>;
    fn store_data<S: Serialize> (&mut self, key:&str, data:&S) -> Result<(), Error>;
    fn load_data<D: DeserializeOwned> (&mut self, key:&str) -> Result<D, Error>;
    fn send_log(&mut self, record:&log::Record);
    fn store_result(&mut self, result:&crate::CheckerResult) -> Result<(), Error>;
}


pub struct IpcControlInterface {
    input: BufReader<File>,
    output: File
}


impl IpcControlInterface {
    pub fn new() -> IpcControlInterface {
        let infile = unsafe {std::fs::File::from_raw_fd(3)} ;
        let outfile = unsafe {std::fs::File::from_raw_fd(4)} ;
        IpcControlInterface { input: BufReader::new(infile),
                              output: outfile }
    }

    fn send<S: Serialize>(&mut self, key:&str, data:&S) -> Result<(), Error> {
        let json = SendMessage {action: key.to_string(), param: data };
        self.output.write(serde_json::to_string(&json)?.as_bytes())?;
        Ok(())
    }


    fn communicate<S: Serialize, D: Clone + DeserializeOwned>(&mut self, key:&str, data:&S) -> Result<D, Error> {
        let mut resp = String::new();
        let json = SendMessage {action: key.to_string(), param: data };

        self.output.write(serde_json::to_string(&json)?.as_bytes())?;
        self.input.read_line(&mut resp)?;

        let r:ReceiveMessage<D> = serde_json::from_reader(resp.as_bytes())?;
        Ok(r.response)
    }
}


impl ControlInterface for IpcControlInterface {
    fn setup() -> Result<(), Error> {
        Ok(())
    }


    fn get_flag(&mut self, tick:u32, payload:&Vec<u8>) -> Result<String, Error> {
        let response:String =
            self.communicate("FLAG", &SendMessageGetFlag { tick: tick, payload: base64::encode(payload.as_slice()) } )?;
        Ok(response)
    }


    fn store_data<S: Serialize> (&mut self, key:&str, data:&S) -> Result<(), Error> {
        let payload = serde_json::to_string(data)?;
        self.send("STORE", &SendMessageStore { key: key.to_string(), data: payload } )?;
        Ok(())
    }


    fn load_data<D: DeserializeOwned> (&mut self, key:&str) -> Result<D, Error> {
        let response:String = self.communicate("LOAD", &key.to_string())?;
        Ok(serde_json::from_reader(response.as_bytes())?)
    }


    fn send_log(&mut self, record:&log::Record) {
        self.send("LOG", &SendMessageLog::from(record)).unwrap()
    }


    fn store_result(&mut self, result:&crate::CheckerResult) -> Result<(), Error> {
        self.send("RESULT", result)?;
        Ok(())
    }
}

impl From<&::log::Record<'_>> for SendMessageLog {
    fn from(record: &::log::Record) -> Self {
        SendMessageLog {
            level: match record.level() {
                ::log::Level::Error => 40,
                ::log::Level::Warn => 30,
                ::log::Level::Info => 20,
                ::log::Level::Debug => 10,
                ::log::Level::Trace => 5,
            },
            message: format!("{}", record.args()),
            funcName: record.module_path().unwrap_or("").to_string(),
            pathname: record.file().unwrap_or("").to_string(),
            lineno: record.line().unwrap_or(0)
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct SendMessageStore {
    key: String,
    data: String
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct SendMessageLog {
    level: u32,
    message: String,
    funcName: String,
    pathname: String,
    lineno: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SendMessageGetFlag {
    payload: String,
    tick: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ReceiveMessage<D> {
    response: D
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SendMessage<D> {
    action: String,
    param: D
}
