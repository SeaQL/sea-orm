use bytes::Bytes;
use flume::Sender;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use sea_orm::ProxyExecResult;
use wasmtime_wasi::preview2::{
    HostInputStream, HostOutputStream, StdinStream, StdoutStream, StreamResult, Subscribe,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RequestMsg {
    Query(String),
    Execute(String),

    Debug(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponseMsg {
    Query(Vec<serde_json::Value>),
    Execute(ProxyExecResult),

    None,
}

pub struct InputStream {
    pub tasks: Arc<Mutex<Vec<ResponseMsg>>>,
}

#[async_trait::async_trait]
impl Subscribe for InputStream {
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl HostInputStream for InputStream {
    fn read(&mut self, _size: usize) -> StreamResult<Bytes> {
        loop {
            {
                let mut tasks = self.tasks.lock().unwrap();
                if tasks.len() > 0 {
                    let ret = tasks.remove(0);
                    let ret = serde_json::to_string(&ret).unwrap() + "\n";
                    let ret = Bytes::from(ret);

                    return Ok(ret);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub struct OutputStream {
    pub tx: Sender<RequestMsg>,
}

#[async_trait::async_trait]
impl Subscribe for OutputStream {
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl HostOutputStream for OutputStream {
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        let msg = String::from_utf8(bytes.to_vec()).expect("Failed to parse message");
        let msg = serde_json::from_str::<RequestMsg>(&msg).expect("Failed to parse message");

        self.tx.send(msg).expect("Failed to send message");
        Ok(())
    }

    fn flush(&mut self) -> StreamResult<()> {
        Ok(())
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        Ok(8192)
    }
}

pub struct HostInputStreamBox {
    pub tasks: Arc<Mutex<Vec<ResponseMsg>>>,
}

impl StdinStream for HostInputStreamBox {
    fn stream(&self) -> Box<dyn HostInputStream> {
        Box::new(InputStream {
            tasks: self.tasks.clone(),
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}

pub struct HostOutputStreamBox {
    pub tx: Sender<RequestMsg>,
}

impl StdoutStream for HostOutputStreamBox {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(OutputStream {
            tx: self.tx.clone(),
        })
    }

    fn isatty(&self) -> bool {
        false
    }
}
