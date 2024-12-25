use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::header::REFERER;
use std::collections::HashMap;
use std::default::Default;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{fs, io, env};
use std::path::Path;
use chrono::{DateTime, Local};
use yansi::Paint;
use indicatif;
use toml::{Value, Table};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateInfo {
    /// 文件列表
    pub filelist: Vec<String>,
    /// 文件列表中第一个文件的Hash，可选
    pub sha1: Option<String>,
    /// URA目录文件列表，可选
    pub filelist_ura: Option<Vec<String>>
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VersionToml {
    /// 配置表
    pub conf: HashMap<String, UpdateInfo>,
    /// 版本
    pub version: HashMap<String, String>
}

impl TryFrom<Table> for VersionToml {
    type Error = anyhow::Error;

    fn try_from(value: Table) -> Result<Self, Self::Error> {
        let mut ret = VersionToml {
            ..Default::default()
        };
        for (k, v) in value.iter() {
            match k.as_str() {
                "version" => ret.version = v.clone().try_into()?,
                _ => {
                    let conf: UpdateInfo = v.clone().try_into()?;
                    ret.conf.insert(k.clone(), conf);
                }
            }
        }
        Ok(ret)
    }
}

impl FromStr for VersionToml {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let table: Table = toml::from_str(s)?;
        let ret = VersionToml::try_from(table)?;
        Ok(ret)
    }
}


// 获取%localappdata%/下的URA存储目录，如有则返回字符串
fn get_ura_path() -> Result<Option<String>> {
    let ura_path = format!("{}/UmamusumeResponseAnalyzer", env::var("LOCALAPPDATA")?);
    if Path::new(&ura_path).exists() {
        Ok(Some(ura_path))
    } else {
        Ok(None)
    }
}

// 获取本地配置文件
fn get_local_conf() -> Result<Option<VersionToml>> {
    let path = "version.toml";
    if Path::new(path).exists() {
        let content = fs::read_to_string(path)?;
        let ret = VersionToml::from_str(&content)?;
        Ok(Some(ret))
    } else {
        Ok(None)
    }
}
/*
fn get_remote_conf() -> Result<Version> {
    let base_url = "https://cdn2.viktorlab.cn/uma";
    let url = format!("{base_url}/version.toml");
    let referer = "https://viktorlab.cn";

    let cli = Client::new();
    let mut resp = cli.get(url).header(REFERER, referer).send()?;

    if resp.status().is_success() {
        let mut content = String::new();
        resp.read_to_string(&mut content)?;
        let value: Version = toml::from_str(&content)?;
        Ok(value)
    } else {
        Err(anyhow!("HTTP error: {}", resp.status()))
    }
}

fn download() -> Result<()> {
    let files = vec!["version", "umaDB.json", "cardDB.json"];
    let base_url = "https://cdn2.viktorlab.cn";
    let base_path = "db";
    let referer = "https://viktorlab.cn";
    let cli = Client::new();

    println!("{}", "正在更新数据...".green());
    for f in &files {
        let resp = cli.get(format!("{base_url}/{f}")).header(REFERER, referer).send()?;
        if resp.status().is_success() {
            let bytes = resp.bytes()?;   // Bytes iterator
            let mut outf = fs::File::create(format!("{base_path}/{f}"))?;
            outf.write_all(&bytes)?;
            println!("下载 {}", f.cyan());
        } else {
            return Err(anyhow!("{} -> {}", f.red(), resp.status().to_string()))
        }
    }
    println!("{}", "更新完毕".green());
    Ok(())
}

fn update_ai(filelist: &Vec<String>) -> Result<()> {
    let base_url = "https://cdn2.viktorlab.cn";
    let base_path = "ai";
    let referer = "https://viktorlab.cn";
    let cli = Client::new();

    println!("{}", "正在更新AI数据...".green());
    let pb = indicatif::ProgressBar::new(filelist.len() as u64);
    
}
*/
fn main() -> Result<()> {
    let path = env::current_dir()?;
    println!("当前工作目录: {:?}", path);

//    let remote_conf = get_remote_conf()?;
    let local_conf = get_local_conf()?;

    println!("{:#?}", local_conf);
    
  //  if local_conf.is_none() || remote_conf.ai.newer_than(local_conf.unwrap().ai) {
  //      update_ai(&remote_conf.ai.filelist);
  //  }
    println!("按回车键退出...");
    let _ = io::stdin().read(&mut [0u8; 1]);    // 只会读size个字节

    Ok(())
}
