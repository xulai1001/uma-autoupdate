use iced::time::Instant;
use iced::widget::{
    center, checkbox, column, container, image, pick_list, row, slider, text, Column
};
use iced::window;
use iced::{
    Alignment, Bottom, Center, Color, ContentFit, Degrees, Element, Fill, Radians,
    Rotation, Subscription, Theme, Font, Settings
};
use iced::font::Family;

use std::default::Default;
use serde::{Deserialize, Serialize};

mod version_toml;
mod version_widget;
use version_toml::*;
use version_widget::*;

pub fn main() -> iced::Result {
    let settings = Settings {
        default_font: Font::with_name("Microsoft YaHei"),
        ..Default::default()
    };
    iced::application("UmaAI 自动更新工具 0.1.0", MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .theme(|_| Theme::TokyoNight)
        .settings(settings)
        .run()
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    OnLoad,
    OnLoadRemote
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct MainWindow {
    pub version_data: VersionData,
    pub widgets: Vec<VersionWidget>,
    pub info_text: String
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

    fn update_impl(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg {
            Message::OnLoad => {
                // 获取本地版本数据
                let local = get_local_conf()?;
                let version_data = VersionData { local, remote: None };
                self.load(version_data);
                self.info_text = "加载远程版本数据...".to_string();
                self.update(Message::OnLoadRemote);
                Ok(())
            }
            Message::OnLoadRemote => {
                let remote = get_remote_conf()?;
                self.load(VersionData { local: self.version_data.local.clone(), remote: Some(remote) });
                self.info_text = "加载完成".to_string();
                Ok(())
            }
        }
    }

    fn update(&mut self, msg: Message) {
        if let Err(e) = self.update_impl(msg) {
            self.info_text = format!("出现错误: {e}");
        }        
    }

    fn subscription(&self) -> Subscription<Message> {
        window::open_events().map(|_| Message::OnLoad)
    }

    fn view(&self) -> Element<Message> {
        let mut widgets: Vec<_> = self.widgets.iter().map(|w| w.view()).collect();
        let info_widget = text!("{}", self.info_text).into();
        let title_widget = text!("UmaAI 自动更新工具")
            .size(24)
            .align_x(Alignment::Center)
            .into();
        widgets.insert(0, title_widget);
        widgets.push(info_widget);
        let column = Column::from_vec(widgets)
            .spacing(10)
            .align_x(Center);
        container(column)
            .padding(10)
            .into()
    }
}
