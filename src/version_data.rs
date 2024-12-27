use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::default::Default;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VersionInfo {
    /// 显示名字
    pub name: String,
    /// 更新时间
    pub date: String,
    /// 文件列表
    pub filelist: Vec<String>,
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
