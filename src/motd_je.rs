use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use crate::utils::{is_valid_ip_or_domain, is_valid_port};
use bytes::BufMut;
use kovi::serde_json;

/// Java 版 MOTD 信息结构体
#[derive(Debug)]
pub struct MotdJavaInfo {
    pub status: String,
    pub host: String,
    pub motd: String,
    pub agreement: i32,
    pub version: String,
    pub online: i32,
    pub max: i32,
    pub sample: Vec<(String, String)>,
    pub favicon: String,
    pub delay: u64,
}

impl MotdJavaInfo {
    pub fn new() -> Self {
        MotdJavaInfo {
            status: "offline".into(),
            host: "".into(),
            motd: "".into(),
            agreement: 0,
            version: "".into(),
            online: 0,
            max: 0,
            sample: Vec::new(),
            favicon: "".into(),
            delay: 0,
        }
    }
}

/// 写入 VarInt 编码
fn put_varint(buf: &mut Vec<u8>, mut val: i32) {
    while (val & !0x7F) != 0 {
        buf.push(((val & 0x7F) | 0x80) as u8);
        val >>= 7;
    }
    buf.push((val & 0x7F) as u8);
}

/// 读取 VarInt
async fn read_varint<S: AsyncReadExt + Unpin>(
    stream: &mut S,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let mut num_read = 0;
    let mut result = 0u32;
    loop {
        let byte = stream.read_u8().await?;
        result |= ((byte & 0x7F) as u32) << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err("VarInt too large".into());
        }
        if (byte & 0x80) == 0 {
            break;
        }
    }
    Ok(result as i32)
}


pub async fn motd_je(
    host: &str,
    port: u16,
) -> Result<MotdJavaInfo, Box<dyn Error + Send + Sync + 'static>> {
    if !is_valid_ip_or_domain(host) || !is_valid_port(port) {
        return Ok(MotdJavaInfo::new());
    }

    let addr = format!("{}:{}", host, port);
    let start = tokio::time::Instant::now();
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await??;
    stream.set_nodelay(true)?;

    // 发送握手包
    let mut handshake = Vec::new();
    put_varint(&mut handshake, 0);                  // Packet ID
    put_varint(&mut handshake, 575);                // 协议版本（1.15.1）
    put_varint(&mut handshake, host.len() as i32);  // 主机地址长度
    handshake.extend(host.as_bytes());              // 主机地址
    handshake.put_u16(port);                        // 端口
    put_varint(&mut handshake, 1);                  // 下一状态：状态查询

    let mut packet = Vec::new();
    put_varint(&mut packet, handshake.len() as i32);
    packet.extend(&handshake);
    stream.write_all(&packet).await?;

    // 发送状态请求包
    let req = [0x01u8, 0x00];
    stream.write_all(&req).await?;

    // 读取响应
    let _packet_len = read_varint(&mut stream).await?;
    let _packet_id = read_varint(&mut stream).await?;
    let json_len = read_varint(&mut stream).await? as usize;

    let mut payload = vec![0u8; json_len];
    stream.read_exact(&mut payload).await?;
    let json_text = String::from_utf8_lossy(&payload);
    let root: serde_json::Value = serde_json::from_str(&json_text)?;

    // 提取数据
    let desc = &root["description"];
    let motd = if desc.is_string() {
        desc.as_str().unwrap().to_string()
    } else if desc["text"].is_string() {
        desc["text"].as_str().unwrap().to_string()
    } else {
        serde_json::to_string(desc)?
    };

    let players = &root["players"];
    let sample = players["sample"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| {
            Some((
                v["id"].as_str()?.to_string(),
                v["name"].as_str()?.to_string(),
            ))
        })
        .collect();

    let version = &root["version"];
    let favicon = root["favicon"].as_str().unwrap_or("").to_string();

    Ok(MotdJavaInfo {
        status: "online".into(),
        host: host.into(),
        motd,
        agreement: version["protocol"].as_i64().unwrap_or(0) as i32,
        version: version["name"].as_str().unwrap_or("").to_string(),
        online: players["online"].as_i64().unwrap_or(0) as i32,
        max: players["max"].as_i64().unwrap_or(0) as i32,
        sample,
        favicon,
        delay: start.elapsed().as_millis() as u64,
    })
}
