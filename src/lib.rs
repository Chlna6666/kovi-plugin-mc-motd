mod motd_be;
mod motd_je;
mod utils;

use kovi::{Message, PluginBuilder as plugin};
use crate::motd_be::motd_be;
use crate::motd_je::motd_je;
use crate::utils::{is_valid_ip_or_domain, is_valid_port};

#[kovi::plugin]
async fn main() {
    plugin::on_msg(|event| async move {
        if let Some(text) = event.borrow_text() {
            if let Some(rest) = text.strip_prefix("motd ") {
                // 支持 [host]:[port] 或者只 host
                let mut parts = rest.splitn(2, ':');
                let host = parts.next().unwrap().trim();
                let port_opt = parts.next().and_then(|p| p.parse::<u16>().ok());

                // 基本校验
                if !is_valid_ip_or_domain(host) ||
                    port_opt.map_or(false, |p| !is_valid_port(p))
                {
                    return event.reply("无效的 IP/域名 或 端口号");
                }

                // 获取并回复
                match fetch_motd(host, port_opt).await {
                    Ok(resp) => {
                        // JE 分支：提取图标并发送图片消息
                        if let Some(b64) = extract_favicon(&resp) {
                            // 构造 Message
                            let mut msg = Message::new();
                            // 先把 text（去掉“图标: ...”那行）发出去
                            let text_only = remove_favicon_line(&resp);
                            msg = msg.add_image(&format!("base64://{}", b64));
                            msg = msg.add_text(text_only);
                            event.reply(msg);
                        } else {
                            // 普通文本回复
                            event.reply(resp);
                        }
                    }
                    Err(err) => event.reply(err),
                }
            }
        }
    });
}

/// 从 fetch_motd 返回的多行字符串中提取以 `图标: ` 开头的 Base64
fn extract_favicon(s: &str) -> Option<String> {
    s.lines()
        .find(|line| line.starts_with("图标: "))
        .and_then(|line| {
            let raw = line["图标: ".len()..].trim();
            raw.strip_prefix("data:image/png;base64,")
                .map(|b64| b64.to_string())
        })
}

/// 删除 `图标: ...` 那一行，保留其余文本
fn remove_favicon_line(s: &str) -> String {
    s.lines()
        .filter(|line| !line.starts_with("图标: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 根据 host/port 做一次 BE/JE 的尝试和回退
async fn fetch_motd(host: &str, port_opt: Option<u16>) -> Result<String, String> {
    // 1. 确定端口，如果没传就用默认
    let mut port = port_opt.unwrap_or(25565);
    // 2. 根据端口预判协议
    let mut try_be = matches!(port, 19132 | 19133);
    // 3. 如果 host 显式带 "be"，强制 BE
    if host.ends_with("be") {
        try_be = true;
    }
    // 5. 准备格式化闭包
    let fmt_be = |info: motd_be::MotdBEInfo| {
        format!(
            "[BE] 状态: {}\nMOTD: {}\n协议版: {}\n版本: {}\n在线/最大: {}/{}\n存档名: {}\n模式: {}\n唯一ID: {}\n延迟: {}ms",
            info.status, info.motd, info.agreement, info.version,
            info.online, info.max, info.level_name, info.game_mode,
            info.server_unique_id, info.delay
        )
    };
    let fmt_je = |info: motd_je::MotdJavaInfo| {
        format!(
            "[JE] 状态: {}\nMOTD: {}\n协议版: {}\n版本: {}\n在线/最大: {}/{}\n在线玩家列表7: {:?}\n图标: {}\n延迟: {}ms",
            info.status, info.motd, info.agreement, info.version,
            info.online, info.max, info.sample, info.favicon,
            info.delay
        )
    };

    // 6. 按顺序尝试
    if try_be {
        match motd_be(host, port).await {
            Ok(be) if be.status == "online" => Ok(fmt_be(be)),
            _ => {
                port = port_opt.unwrap_or(25565);
                match motd_je(host, port).await {
                    Ok(je) if je.status == "online" => Ok(fmt_je(je)),
                    _ => Err("BE/JE 都无法获取 MOTD 信息".into()),
                }
            }
        }
    } else {
        match motd_je(host, port).await {
            Ok(je) if je.status == "online" => Ok(fmt_je(je)),
            _ => {
                port = port_opt.unwrap_or(19132);
                match motd_be(host, port).await {
                    Ok(be) if be.status == "online" => Ok(fmt_be(be)),
                    _ => Err("JE/BE 都无法获取 MOTD 信息".into()),
                }
            }
        }
    }
}
