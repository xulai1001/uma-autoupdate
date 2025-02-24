#![windows_subsystem = "windows"]
use iced::widget::{
    column, container, image, text, Column
};
use iced::{window, Size};
use iced::{
    Top, Bottom, Center, Element, Fill,
    FillPortion, Subscription, Theme, Font, Settings, Task
};
use iced::advanced::image::Handle;

use futures_channel::mpsc::Sender;
use std::default::Default;
use std::collections::HashMap;
use log::{info, error};
use tokio::runtime::Runtime;
use rust_embed::Embed;

mod modal;
mod version_toml;
mod version_widget;
mod download;
mod utils;

use utils::*;
use modal::*;
use version_toml::*;
use version_widget::*;
use download::*;

type WindowSettings = iced::window::Settings;

pub fn main() -> iced::Result {
    init_logger().expect("logger error");
    // 设置图标
    let icon = window::icon::from_file_data(
        &Res::get("umaai-sm.ico")
            .expect("Icon resource error")
            .data
            .to_vec(),
        None
    ).expect("Icon error");
    // app设定
    let settings = Settings {
        default_font: Font::with_name("Microsoft YaHei"),
        antialiasing: true,
        ..Default::default()
    };
    // 删除旧版
    let _ = remove_old();

    iced::application("UmaAI 自动更新工具 0.1.1", MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .theme(|_| Theme::CatppuccinLatte)
        .settings(settings)
        .window(WindowSettings {
            size: Size { width: 840.0, height: 420.0 },
            icon: Some(icon),   // 图标属于window设定
            ..Default::default()
        })
        .resizable(false)
        .run()
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    OnLoad,
    OnLoadRemote(HashMap<String, VersionInfo>),
    OnSetInfo(String),
    OnClickUpdate(VersionWidget),
    OnDownloadCompleted(DownloadFile),
    OnListenerReady(Sender<Message>)
}

impl Message {
    pub fn text(s: &str) -> Self {
        Message::OnSetInfo(s.to_string())
    }
}

#[derive(Embed)]
#[folder = "res"]
struct Res;

#[derive(Debug, Default, Clone)]
struct MainWindow {
    pub version_data: VersionData,
    pub widgets: Vec<VersionWidget>,
    pub info_text: String,
    /// 发送到下载线程的文件信息下行通道，rx端在下载线程里
    pub tx_file: Option<Sender<DownloadFile>>,
    pub in_progress: HashMap<String, usize>
}

impl MainWindow {
    pub fn load(&mut self, version_data: VersionData) -> &mut Self {
        match version_data.pick() {
            Some(data) => {
                // 重新生成widgets 并按index排序
                let mut widgets = vec![];
                for k in data.keys() {
                    widgets.push(VersionWidget::new(&version_data, k));
                }
                widgets.sort_by_key(|w| w.index);
                self.version_data = version_data;
                self.widgets = widgets;
            }
            None => {
                self.version_data = VersionData::default();
                self.widgets = vec![];
            }
        }
        self
    }

    fn update_impl(&mut self, msg: Message) -> anyhow::Result<Task<Message>> {
        match msg {
            Message::OnLoad => {
                // 获取本地版本数据
                let local = get_local_conf()?;
                let version_data = VersionData { local, remote: None };
                self.load(version_data);
                self.info_text = "加载远程版本数据...".to_string();
                Ok(Task::future(async move {
                    let runtime = Runtime::new().unwrap();
                    runtime.block_on(async move {
                        match get_remote_conf().await {
                            Ok(remote) => {
                                Message::OnLoadRemote(remote)
                            }
                            Err(e) => {
                                Message::OnSetInfo(e.to_string())
                            }
                        }
                    })
                }))
            }
            Message::OnLoadRemote(remote) => {
                self.load(VersionData { local: self.version_data.local.clone(), remote: Some(remote) });
                self.info_text = "加载完成".to_string();
                Ok(Task::none())
            }
            Message::OnSetInfo(text) => {
                info!("{text}");
                self.info_text = text;
                Ok(Task::none())
            }
            Message::OnClickUpdate(widget) => {
                if let Some(remote) = &widget.remote {
                    if let Some(tx_file) = &mut self.tx_file {
                        for filename in &remote.filelist {
                            tx_file.start_send(DownloadFile { key: widget.key.clone(), filename: filename.clone() }).unwrap();
                        }
                        self.in_progress.insert(widget.key.clone(), remote.filelist.len());
                    }
                }
                Ok(Task::done(Message::text(&format!("正在更新 {}", widget.name))))
            }
            Message::OnListenerReady(sender) => {
                // listener已经启动，sender为下载线程使用的消息发动端
                // 这时初始化下载线程，直接使用channel作为下载队列
                let (tx_file, rx_file) = futures_channel::mpsc::channel(128);
                self.tx_file = Some(tx_file);
                let mut worker = DownloadWorker::new(sender, rx_file);
                Ok(Task::perform(async move {
                    worker.run();
                }, |_| { Message::text("listenerready") }))
            }
            Message::OnDownloadCompleted(d) => {
                self.in_progress
                    .entry(d.key.clone())
                    .and_modify(|count| *count -= 1);
                if self.in_progress[&d.key] == 0 {
                    // 下载完成
                    let w = self.widgets
                        .clone()
                        .into_iter()
                        .find(|w| w.key == d.key)
                        .expect("widget not found");
                    if w.verify() {
                        self.version_data.update_and_save(&d.key)?;
                        self.load(self.version_data.clone());
                        if d.key == "auto_update" {
                            replace_self()?;
                        } else {
                            w.replace()?;
                        }
                    } else {
                        let err = format!("{} 更新文件校验错误，请联系管理员", w.key);
                        return Ok(Task::done(Message::text(&err)));
                    }
                }
                Ok(Task::done(Message::text(&format!("下载完成 - {}", d.filename))))
            }
            _ => Ok(Task::none())
        }
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match self.update_impl(msg) {
            Ok(task) => task,
            Err(e) => {
                self.info_text = format!("出现错误: {e}");
                error!("{}", self.info_text);
                Task::none()
            }
        }        
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::open_events().map(|_| Message::OnLoad),
            Subscription::run(download::listener)
        ])
    }

    fn view(&self) -> Element<Message> {
        let background = container(
                image(Handle::from_bytes(
                    Res::get("umaai-1.jpg")
                        .expect("Resource error")
                        .data
                        .to_vec()
                )).width(Fill)
                .height(Fill)
            ).width(Fill)
            .height(Fill);

        let widgets: Vec<_> = self.widgets
            .iter()
            .map(|w| w.view())
            .collect();
        let info_widget = text!("{}", self.info_text)
            .align_x(Center)
            .align_y(Bottom)
            .height(FillPortion(1));
        let title_widget = text!("UmaAI 自动更新工具")
            .size(24)
            .align_x(Center)
            .align_y(Center)
            .height(FillPortion(1));
        
        let column = column![
            title_widget,
            Column::from_vec(widgets)
                .width(Fill)
                .height(FillPortion(6)),
            info_widget
        ].spacing(6)
        .align_x(Center)        
        .height(Fill);

        let element: Element<Message> = container(column)
            .height(Fill)
            .width(Fill)
            .align_x(Center)
            .align_y(Center)
            .padding(6)
            .into();
        //element// .explain(Color::from_rgb(0.0, 1.0, 0.0))
        Modal::new(background, element).into()
    }
}
