// 本文件基于 [MCBE-Server-Motd](https://github.com/BlackBEDevelopment/MCBE-Server-Motd/) 从 Go 转换至 Rust
// 原始代码版权所有 (c) BlackBEDevelopment - 遵循 MPL-2.0 协议
// 转换日期：2025-07-22
// 转换工具：AI 辅助工具
//
// 此文件遵循 Mozilla Public License 2.0 (MPL-2.0)
// 完整协议文本：https://mozilla.org/MPL/2.0/

use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use byteorder::{BigEndian, WriteBytesExt}; 
use std::error::Error;
use crate::utils::{is_valid_ip_or_domain, is_valid_port};

#[derive(Debug)]
pub struct MotdBEInfo {
    pub status: String,
    pub host: String,
    pub motd: String,
    pub agreement: u32,
    pub version: String,
    pub online: u32,
    pub max: u32,
    pub level_name: String,
    pub game_mode: String,
    pub server_unique_id: String,
    pub delay: u64,
}

impl MotdBEInfo {
    pub fn new() -> Self {
        MotdBEInfo {
            status: "offline".to_string(),
            host: "".to_string(),
            motd: "".to_string(),
            agreement: 0,
            version: "".to_string(),
            online: 0,
            max: 0,
            level_name: "".to_string(),
            game_mode: "".to_string(),
            server_unique_id: "".to_string(),
            delay: 0,
        }
    }
}

pub async fn motd_be(
    host: &str,
    port: u16,
) -> Result<MotdBEInfo, Box<dyn Error + Send + Sync + 'static>> {
    if host.is_empty() {
        return Ok(MotdBEInfo::new());
    }

    // 检查 IP 和端口是否合法
    if !is_valid_ip_or_domain(host) || !is_valid_port(port) {
        return Ok(MotdBEInfo::new());
    }

    // 通过提供的 host 和 port 建立 UDP 连接
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let addr = format!("{}:{}", host, port);
    socket.connect(&addr).await?;

    let mut send_data = vec![
        0x01, // Packet ID
    ];

    let timestamp = tokio::time::Instant::now().elapsed().as_millis() as u64;
    let mut timestamp_bytes = Vec::new();
    timestamp_bytes.write_u64::<BigEndian>(timestamp)?;

    let magic = vec![0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD];
    let client_id = vec![0x12, 0x34, 0x56, 0x78];
    let client_guid = vec![0; 8];

    send_data.extend(timestamp_bytes);
    send_data.extend(magic);
    send_data.extend(client_id);
    send_data.extend(client_guid);

    socket.send(&send_data).await?;

    let mut buffer = vec![0; 1024];
    let start_time = tokio::time::Instant::now();

    let result = timeout(Duration::from_secs(5), socket.recv_from(&mut buffer)).await;

    match result {
        Ok(Ok((len, _))) => {
            let data = &buffer[..len];
            let motd_data = String::from_utf8_lossy(data);
            let parts: Vec<&str> = motd_data.split(';').collect();

            if parts.len() >= 9 {
                return Ok(MotdBEInfo {
                    status: "online".to_string(),
                    host: host.to_string(),
                    motd: parts[1].to_string(),
                    agreement: parts[2].parse().unwrap_or(0),
                    version: parts[3].to_string(),
                    online: parts[4].parse().unwrap_or(0),
                    max: parts[5].parse().unwrap_or(0),
                    level_name: parts[7].to_string(),
                    game_mode: parts[8].to_string(),
                    server_unique_id: parts[6].to_string(),
                    delay: start_time.elapsed().as_millis() as u64,
                });
            }
            Err("响应数据无效".into())
        }
        _ => {
            Ok(MotdBEInfo::new()) // 超时或错误，返回默认 "offline" 信息
        }
    }
}
