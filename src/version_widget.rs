//! version_info  
//! 显示单个app版本信息的组件
use crate::version_toml::*;
use crate::Message;
use anyhow::{anyhow, Result};
use iced::widget::{button, container, row, text};
use iced::{Center, Color, Element, Fill, FillPortion, Shadow, Vector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionWidget {
    pub key: String,
    pub name: String,
    pub index: u32,
    pub local: Option<VersionInfo>,
    pub remote: Option<VersionInfo>,
}

fn get_update_time(opt: &Option<VersionInfo>) -> String {
    opt.as_ref()
        .and_then(|v| v.date.split(".").nth(0))
        .map(|st| st.replace(" ", "\n"))
        .unwrap_or("无".to_string())
}

macro_rules! def_align {
    ($elem:expr, $width:expr) => {
        $elem
            .height(Fill)
            .align_x(Center)
            .align_y(Center)
            .width(FillPortion($width))
    };
}

fn bg_style(color: Color) -> container::Style {
    container::Style {
        background: Some(color.into()),
        //border: Border {
        //    radius: 10.into(),
        //    ..Default::default()
        //},
        //shadow: Shadow {
        //    offset: Vector::new(3.0, 3.0),
        //    color: Color::from_rgb8(96, 96, 96),
        //    blur_radius: 0.0
        //},
        ..Default::default()
    }
}

fn btn_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let mut sty = button::primary(theme, status);
    sty.shadow = Shadow {
        offset: Vector::new(3.0, 3.0),
        color: Color::from_rgb8(96, 96, 96),
        blur_radius: 0.0,
    };
    sty
}

impl VersionWidget {
    pub fn new(data: &VersionData, key: &str) -> Self {
        let local = data
            .local
            .as_ref()
            .and_then(|local| local.get(key))
            .cloned();
        let remote = data
            .remote
            .as_ref()
            .and_then(|remote| remote.get(key))
            .cloned();
        let pick = remote.as_ref().or(local.as_ref());
        let index = pick.map(|v| v.index).unwrap_or(0);
        let name = pick.map(|v| v.name.clone()).unwrap_or(key.to_string());
        Self {
            key: key.to_string(),
            name,
            index,
            local,
            remote,
        }
    }

    pub fn needs_update(&self) -> bool {
        match (&self.local, &self.remote) {
            (Some(local), Some(remote)) => remote.newer_than(local),
            (None, Some(_)) => true,
            _ => false,
        }
    }
    pub fn view(&self) -> Element<Message> {
        let name = text(&self.name)
            .size(20)
            .width(FillPortion(3))
            .height(Fill)
            .align_y(Center);

        let local_row = def_align!(
            container(text!("本地版本: {}", get_update_time(&self.local))),
            3
        )
        .style(|_| bg_style(Color::from_rgba8(128, 255, 128, 0.85)));

        let remote_row = def_align!(
            container(text!("最新版本: {}", get_update_time(&self.remote))),
            3
        )
        .style(|_| bg_style(Color::from_rgba8(192, 0, 255, 0.85)));

        let on_press_msg = if self.needs_update() {
            Some(Message::OnClickUpdate(self.clone()))
        } else {
            None
        };

        let btn_update = button(text("更新").color(Color::WHITE).align_y(Center))
            .style(button::primary)
            .padding([32, 40])
            .on_press_maybe(on_press_msg)
            .height(Fill);

        container(row![name, local_row, remote_row, btn_update].spacing(20))
            .padding(5)
            .into()
    }

    pub fn verify(&self) -> bool {
        self.remote.as_ref().map(|x| x.verify()).unwrap_or(false)
    }
    pub fn replace(&self) -> Result<()> {
        self.remote
            .as_ref()
            .map(|x| x.install())
            .unwrap_or(Err(anyhow!("未获取远程版本")))
    }
}
