use anyhow::Result;
use bytes::Bytes;
use flume::Sender;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use wasmtime_wasi::preview2::{HostInputStream, HostOutputStream, OutputStreamError, StreamState};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub id: u32,
    pub data: String,
}

pub struct InputStream {
    pub tasks: Arc<Mutex<Vec<Msg>>>,
}

#[async_trait::async_trait]
impl HostInputStream for InputStream {
    fn read(&mut self, _size: usize) -> Result<(Bytes, StreamState)> {
        loop {
            {
                let mut tasks = self.tasks.lock().unwrap();
                if tasks.len() > 0 {
                    let ret = tasks.remove(0);
                    let ret = ron::to_string(&ret).unwrap() + "\n";
                    let ret = Bytes::from(ret);

                    return Ok((ret, StreamState::Open));
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    async fn ready(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct OutputStream {
    pub tx: Sender<Msg>,
}

#[async_trait::async_trait]
impl HostOutputStream for OutputStream {
    fn write(&mut self, bytes: Bytes) -> Result<(), OutputStreamError> {
        let msg =
            String::from_utf8(bytes.to_vec()).map_err(|e| OutputStreamError::Trap(e.into()))?;
        let msg = ron::from_str::<Msg>(&msg).map_err(|e| OutputStreamError::Trap(e.into()))?;

        self.tx
            .send(msg)
            .map_err(|e| OutputStreamError::Trap(e.into()))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), OutputStreamError> {
        Ok(())
    }

    async fn write_ready(&mut self) -> Result<usize, OutputStreamError> {
        Ok(8192)
    }
}
