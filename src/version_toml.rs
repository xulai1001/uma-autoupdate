use crate::utils::*;
use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime};
use log::{info, warn};
use reqwest::{header::REFERER, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VersionInfo {
    /// 显示名字
    pub name: String,
    /// 更新时间
    pub date: String,
    /// 显示顺序
    pub index: u32,
    /// 文件列表
    pub filelist: Vec<String>,
    /// 文件列表中第一个文件的Hash，实际不可空
    pub sha1: Option<String>,
    /// 完整压缩包名，可选
    pub package: Option<String>,
    /// 完整压缩包Hash
    pub package_sha1: Option<String>,
    /// App版本说明，可选
    pub ver: Option<String>,
    /// 下载目录，可选
    pub install_path: Option<String>,
}

impl VersionInfo {
    pub fn newer_than(&self, other: &VersionInfo) -> bool {
        let date1 = NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%d %H:%M:%S");
        let date2 = NaiveDateTime::parse_from_str(&other.date, "%Y-%m-%d %H:%M:%S");
        match (date1, date2) {
            (Ok(d1), Ok(d2)) => d1 > d2,
            (Ok(_), Err(_)) => true,
            _ => false,
        }
    }

    /// 检查filelist的第一个文件的sha1是否为记录的值
    pub fn verify(&self) -> bool {
        if let Some(sha1) = self.sha1.as_deref() {
            let filename = format!("{}.autoupdate", self.filelist[0]);
            match get_file_sha1(&filename) {
                Ok(digest) => {
                    if digest != sha1 {
                        warn!("{filename} SHA1错误");
                    }
                }
                Err(e) => {
                    warn!("计算{filename} SHA1时出错: {e:?}");
                }
            } 
        }
        true
    }

    pub fn get_install_dir(&self) -> Result<String> {
        let mut install_path = self.install_path.as_deref().unwrap_or(".").to_string();
        if install_path.contains("%localappdata%") {
            let data_path = env::var("LOCALAPPDATA")?;
            info!("local-app-data: {data_path}");
            install_path = install_path.replace("%localappdata%", &data_path);
        }
        Ok(install_path)
    }

    pub fn get_local_sha1(&self) -> Result<Option<String>> {
        let install_dir = self.get_install_dir()?;
        // 拿name判断一下, emm...
        let local_name = if self.name == "自动更新工具" {
            get_exe_name()?.replace(".\\", "")
        } else {
            self.filelist[0].clone()
        };
        let filename = format!("{}/{}", install_dir, &local_name);
        if fs::exists(&filename)? {
            let digest = get_file_sha1(&filename)?;
            Ok(Some(digest))
        } else {
            info!("本地文件 {local_name} 不存在.");
            Ok(None)
        }
    }

    pub fn install(&self) -> Result<()> {
        let install_path = self.get_install_dir()?;
        if !fs::exists(&install_path)? {
            info!("新建目录 {install_path}");
            fs::create_dir_all(&install_path)?;
        }
        for filename in &self.filelist {
            let tempfile = format!("{filename}.autoupdate");
            if fs::exists(&tempfile)? {
                let new_file = PathBuf::from(&install_path).join(filename);
                info!("Copy {tempfile} -> {new_file:?}");
                // windows的限制，只能复制+删除，不能rename
                fs::copy(&tempfile, &new_file)?;
                fs::remove_file(&tempfile)?;
            }
        }
        Ok(())
    }
}

type VersionToml = HashMap<String, VersionInfo>;

/// 提供给前端的的版本信息集合  
/// (可能获取不到)可以为空
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VersionData {
    /// 本地版本信息
    pub local: Option<VersionToml>,
    /// 远程版本信息
    pub remote: Option<VersionToml>,
}

impl VersionData {
    /// 按remote - local - None的顺序返回一个有效配置
    pub fn pick(&self) -> Option<&VersionToml> {
        self.remote.as_ref().or(self.local.as_ref())
    }

    pub fn update_and_save(&mut self, key: &str) -> Result<()> {
        let mut local = self.local.clone().unwrap_or_default();
        if let Some(remote) = self.remote.as_ref() {
            if let Some(info) = remote.get(key) {
                local.insert(key.to_string(), info.clone());
            }
            info!("update version.toml: {key}");
            let mut file = fs::File::create("version.toml")?;
            file.write_all(toml::to_string_pretty(&local)?.as_bytes())?;
            self.local = Some(local);
        }
        Ok(())
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
pub async fn get_remote_conf() -> Result<VersionToml> {
    let base_url = "https://cdn2.viktorlab.cn/uma";
    let url = format!("{base_url}/version.toml");
    let referer = "https://viktorlab.cn";
    let cli = Client::new();
    let resp = cli.get(url).header(REFERER, referer).send().await?;

    if resp.status().is_success() {
        let content = String::from_utf8(resp.bytes().await?.to_vec())?;
        let ret: VersionToml = toml::from_str(&content)?;
        Ok(ret)
    } else {
        Err(anyhow!(resp.status().to_string()))
    }
}

pub async fn get_version_data() -> Result<VersionData> {
    let local = get_local_conf()?;
    let remote = get_remote_conf().await.ok();
    Ok(VersionData { local, remote })
}

#[cfg(test)]
#[tokio::test]
async fn test_version_toml() -> Result<()> {
    let path = env::current_dir()?;
    println!("当前工作目录: {:?}", path);

    let local_conf = get_local_conf()?;
    println!("Local: {:#?}", local_conf);

    let remote_conf = get_remote_conf().await?;
    println!("Remote: {remote_conf:#?}");
    Ok(())
}

#[cfg(test)]
#[test]
fn test_date() -> Result<()> {
    let date1 = NaiveDate::parse_from_str("2024-05-05", "%Y-%m-%d");
    let date2 = NaiveDate::parse_from_str("2024-03-05", "%Y-%m-%d");
    let res = match (date1, date2) {
        (Ok(d1), Ok(d2)) => d1 > d2,
        (Ok(_), Err(_)) => true,
        _ => false,
    };
    println!("{date1:?}, {date2:?}, d1 > d2 = {res}");
    assert!(res);
    Ok(())
}
