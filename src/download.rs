use anyhow::Result;
use futures_core::stream::Stream;
use reqwest::Client;
use reqwest::header::REFERER;
use serde::{Deserialize, Serialize};
use futures_channel::mpsc::{Sender, Receiver};
use futures_util::{SinkExt, StreamExt};
use std::fs::File;
use std::io::Write;
use crate::Message;

/// 下载线程的消息监听线程  
/// 使用iced_futures::stream::channel构建Stream并交给subscription
pub fn listener() -> impl Stream<Item = Message> {
    iced::stream::channel(1024, |mut output| async move {
        // 实际传递消息的是tokio mpsc channel ??
        let (tx, mut rx) = futures_channel::mpsc::channel(8);
        // 把tx端发送给主线程
        output.send(Message::OnListenerReady(tx)).await.unwrap();
        loop {
            // rx端从tokio channel收到消息，再转发到stream channel里
            if let Some(msg) = rx.next().await {
                output.send(msg).await.unwrap();
            }
        }
    })
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct DownloadFile {
    pub key: String,
    pub filename: String
}

impl DownloadFile {
    pub fn tempfile(&self) -> String {
        format!("{}.autoupdate", self.filename)
    }

    pub fn url(&self) -> String {
        format!("https://cdn2.viktorlab.cn/uma/{}/{}", self.key, self.filename)
    }
}

pub struct DownloadWorker {
    cli: Client,
    /// 用于向主线程发送消息
    channel: Sender<Message>,
    /// 用于接收要下载的文件信息
    control: Receiver<DownloadFile>
}

impl DownloadWorker {
    pub fn new(channel: Sender<Message>, control: Receiver<DownloadFile>) -> Self {
        DownloadWorker {
            cli: Client::new(),
            channel,
            control
        }
    }
    pub async fn run_guarded(&mut self) -> Result<()> {
        while let Some(file) = self.control.next().await {
            let resp = self.cli.get(file.url())
                .header(REFERER, "https://viktorlab.cn")
                .send().await?;
            let total_size = resp.content_length().unwrap_or(1) as usize;
            let mut stream = resp.bytes_stream();

            let mut temp_file = File::create(file.tempfile())?;
            let mut downloaded_size = 0;
            let mut last_progress = 0;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                temp_file.write_all(&chunk)?;
                downloaded_size += chunk.len();
                // 减少消息数量
                if downloaded_size - last_progress > 128000 {
                    self.channel.start_send(
                        Message::OnSetInfo(format!(
                            "下载 [{}]{} ({} / {})",
                            file.key,
                            file.filename,
                            downloaded_size,
                            total_size
                        ))
                    )?;
                    last_progress = downloaded_size;
                }
            }
            self.channel.start_send(Message::OnDownloadCompleted(file))?;
        }        
        Ok(())
    }

    #[tokio::main]
    pub async fn run(&mut self) {
        loop {
            // 正常退出则返回，异常时报错并重试，避免因为线程内部异常导致线程退出
            match self.run_guarded().await {
                Ok(_) => break,
                Err(e) => {
                    self.channel
                        .send(Message::OnSetInfo(format!("下载失败: {e}")))
                        .await.unwrap();
                }
            }
        }
    }
}
