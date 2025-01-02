//! version_info  
//! 显示单个app版本信息的组件

use iced::time::Instant;
use iced::widget::{
    center, checkbox, column, container, image, pick_list, row, slider, text,
};
use iced::window;
use iced::{
    Bottom, Center, Color, ContentFit, Degrees, Element, Fill, Radians,
    Rotation, Subscription, Theme
};
use serde::{Deserialize, Serialize};
use crate::version_toml::*;
use crate::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionWidget {
    pub name: String,
    pub index: u32,
    pub local: Option<VersionInfo>,
    pub remote: Option<VersionInfo>
}

fn get_update_time(opt: &Option<VersionInfo>) -> &str {
    opt.as_ref().map(|v| v.date.as_str()).unwrap_or("无")
}

impl VersionWidget {
    pub fn new(data: &VersionData, name: &str) -> Self {
        let local = data.local
            .as_ref()
            .and_then(|local| local.get(name))
            .cloned();
        let remote = data.remote
            .as_ref()
            .and_then(|remote| remote.get(name))
            .cloned();
        let pick = remote.as_ref().or(local.as_ref());
        let index = pick.map(|v| v.index).unwrap_or(0);
        let name = pick.map(|v| v.name.clone()).unwrap_or(name.to_string());
        Self { name, index, local, remote }
    }

    pub fn update(&mut self, msg: Message) {

    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<Message> {
        let name = text(&self.name).size(20);
        container(row![
            name,
            "本地版本",
            get_update_time(&self.local),
            "最新版本",
            get_update_time(&self.remote),
            "更新"
        ].spacing(10))
        .padding(10)
        .into()
    }
}
