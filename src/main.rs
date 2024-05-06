use reqwest::blocking::Client;
use reqwest::header::REFERER;
use std::io::{Read, Write};
use std::error::Error;
use std::{fs, io, env};
use std::io::ErrorKind;
use std::path::Path;
use chrono::{DateTime, FixedOffset};
use yansi::Paint;

// Box<dyn Error>可以返回任何错误，属于使用指针的运行时多态，而且具有所有权
fn get_local_ver() -> Result<i64, Box<dyn Error>> {
    let path = "db/version";
    if Path::new(path).exists() {
        let line = fs::read_to_string(path)?;   // line is String
        let datetime = line.parse::<DateTime<FixedOffset>>()?; // turbofish annotation
        println!("本地: {}", datetime);
        let timestamp = datetime.timestamp();
        Ok(timestamp)
    } else {
        // ?对应Try trait，Err的Try trait调用From trait对异常类型进行自动转换，把括号里的内容装箱变成Box<Error>
        // 相当于Err(Box::from(Error::new(x)))省略了装箱过程
        Err(io::Error::new(ErrorKind::NotFound, path))?
    }
}

fn get_remote_ver() -> Result<i64, Box<dyn Error>> {
    let base_url = "https://cdn2.viktorlab.cn";
    let url = format!("{base_url}/version");
    let referer = "https://viktorlab.cn";

    let cli = Client::new();
    let mut resp = cli.get(url).header(REFERER, referer).send()?;

    if resp.status().is_success() {
        let mut content = String::new();
        resp.read_to_string(&mut content)?;
        let datetime: DateTime<FixedOffset> = content.parse()?;
        println!("最新: {}", datetime);
        Ok(datetime.timestamp())
    } else {
        Err(resp.status().to_string())? // 同理，Err?把String转为Error并装箱，最终变成Box<Error>了
    }
}

fn download() -> Result<(), Box<dyn Error>> {
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
            Err(format!("{} -> {}", f.red(), resp.status().to_string()))?
        }
    }
    println!("{}", "更新完毕".green());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::current_dir()?;
    println!("当前工作目录: {}", path.display());

    let remote_ver = get_remote_ver()?;
    let local_ver = get_local_ver().unwrap_or(0);
    if local_ver < remote_ver {
        download()?;
    } else {
        println!("{}", "已经是最新版本".green());
    }
    println!("按回车键退出...");
    let _ = io::stdin().read(&mut [0u8; 1]);    // 只会读size个字节
    Ok(())
}
