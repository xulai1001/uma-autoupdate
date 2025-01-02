use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::default::Default;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{fs, io, env};
use std::path::Path;
use chrono::{DateTime, Local};
use toml::{Value, Table};
use serde::{Deserialize, Serialize};
use reqwest::{blocking::Client, header::REFERER};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VersionInfo {
    /// 显示名字
    pub name: String,
    /// 更新时间
    pub date: String,
    /// 文件列表
    pub filelist: Vec<String>,
    /// 显示顺序
    pub index: u32,
    /// URA目录文件列表，可选
    pub filelist_ura: Option<Vec<String>>,
    /// 文件列表中第一个文件的Hash，可选
    pub sha1: Option<String>,
    /// App版本号，可选
    pub ver: Option<String>
}

type VersionToml = HashMap<String, VersionInfo>;

/// 提供给前端的的版本信息集合  
/// (可能获取不到)可以为空
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VersionData {
    /// 本地版本信息
    pub local: Option<VersionToml>,
    /// 远程版本信息
    pub remote: Option<VersionToml>
}

impl VersionData {
    /// 按remote - local - None的顺序返回一个有效配置
    pub fn pick(&self) -> Option<&VersionToml> {
        self.remote.as_ref()
            .or(self.local.as_ref())
    }
}

/// 获取本地配置文件
pub fn get_local_conf() -> Result<Option<VersionToml>> {
    let path = "version.toml";
    if Path::new(path).exists() {
        let content = fs::read_to_string(path)?;
        let ret: VersionToml = toml::from_str(&content)?;
        Ok(Some(ret))
    } else {
        Ok(None)
    }
}

/// 获取远程配置文件
pub fn get_remote_conf() -> Result<VersionToml> {
    let base_url = "https://cdn2.viktorlab.cn/uma";
    let url = format!("{base_url}/version.toml");
    let referer = "https://viktorlab.cn";

    let cli = Client::new();
    let mut resp = cli.get(url).header(REFERER, referer).send()?;

    if resp.status().is_success() {
        let mut content = String::new();
        resp.read_to_string(&mut content)?;
        let ret: VersionToml = toml::from_str(&content)?;
        Ok(ret)
    } else {
        Err(anyhow!(resp.status().to_string()))
    }
}

pub fn get_version_data() -> Result<VersionData> {
    let local = get_local_conf()?;
    let remote = get_remote_conf().ok();
    Ok(VersionData { local, remote })
}

#[cfg(test)]
#[test]
fn test_version_toml() -> Result<()> {
    let path = env::current_dir()?;
    println!("当前工作目录: {:?}", path);

    let local_conf = get_local_conf()?;
    println!("Local: {:#?}", local_conf);

    let remote_conf = get_remote_conf()?;
    println!("Remote: {remote_conf:#?}");
    Ok(())
}