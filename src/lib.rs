mod motd_be;
mod motd_je;
mod utils;

use kovi::{Message, PluginBuilder as plugin};
use kovi::log;
use crate::motd_be::motd_be;
use crate::motd_je::motd_je;
use crate::utils::{is_valid_ip_or_domain, is_valid_port};
use trust_dns_resolver::{TokioAsyncResolver, config::{ResolverConfig, ResolverOpts}};

/// 异步构造 DNS 解析器，用于 SRV 查询
async fn make_resolver() -> TokioAsyncResolver {
    let opts = ResolverOpts::default();
    let config = ResolverConfig::default();
    TokioAsyncResolver::tokio(config, opts)
}

#[kovi::plugin]
async fn main() {
    plugin::on_msg(|event| async move {
        if let Some(text) = event.borrow_text() {
            if let Some(rest) = text.strip_prefix("motd ") {
                let mut parts = rest.splitn(2, ':');
                let host = parts.next().unwrap().trim();
                let port_opt = parts.next().and_then(|p| p.parse::<u16>().ok());

                // 基本校验
                if !is_valid_ip_or_domain(host) ||
                    port_opt.map_or(false, |p| !is_valid_port(p)) {
                    return event.reply("无效的 IP/域名 或 端口号");
                }

                // 调用主逻辑并返回结果
                match fetch_motd(host, port_opt).await {
                    Ok(resp) => {
                        if let Some(b64) = extract_favicon(&resp) {
                            let mut msg = Message::new();
                            let text_only = remove_favicon_line(&resp);
                            msg = msg.add_image(&format!("base64://{}", b64));
                            msg = msg.add_text(text_only);
                            event.reply(msg);
                        } else {
                            event.reply(resp);
                        }
                    }
                    Err(err) => event.reply(err),
                }
            }
        }
    });
}

/// 提取 Base64 图标，去除前缀
fn extract_favicon(s: &str) -> Option<String> {
    s.lines()
        .find(|line| line.starts_with("图标: "))
        .and_then(|line| {
            let raw = &line["图标: ".len()..];
            raw.trim().strip_prefix("data:image/png;base64,")
                .map(|b| b.to_string())
        })
}

/// 删除图标行，仅保留文本
fn remove_favicon_line(s: &str) -> String {
    s.lines()
        .filter(|line| !line.starts_with("图标: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 尝试 SRV 解析，再根据协议顺序落地 BE/JE
async fn fetch_motd(host: &str, port_opt: Option<u16>) -> Result<String, String> {
    log::info!("正在获取 MOTD: 主机={} 端口={:?}", host, port_opt);
    let resolver = make_resolver().await;

    // 初始端口与协议判定
    let mut port = port_opt.unwrap_or(25565);
    let mut try_be = matches!(port, 19132 | 19133);
    if host.ends_with("be") {
        try_be = true;
    }
    log::debug!("初始设置 -> 端口={} 优先 BE={}", port, try_be);

    // SRV 查询（未指定端口时）
    if port_opt.is_none() {
        let srv_name = format!("_minecraft._tcp.{}", host);
        log::info!("执行 SRV 查询: {}", srv_name);
        match resolver.srv_lookup(srv_name.clone()).await {
            Ok(lookup) => {
                log::debug!("SRV 查询到 {} 条记录", lookup.iter().count());
                if let Some(rec) = lookup.iter().next() {
                    let raw_target = rec.target().to_utf8();
                    let je_host = raw_target.trim_end_matches('.');
                    let je_port = rec.port();
                    log::info!("SRV 记录 -> 原始:{} 清理后:{} 端口:{}", raw_target, je_host, je_port);
                    if let Ok(je) = motd_je(je_host, je_port).await {
                        if je.status == "online" {
                            log::info!("通过 SRV 查询的 JE 服务器在线");
                            return Ok(fmt_je(je));
                        } else {
                            log::warn!("JE 状态: {}", je.status);
                        }
                    } else {
                        log::error!("JE 查询错误 via SRV");
                    }
                }
            }
            Err(e) => log::error!("SRV 查询失败: {}", e),
        }
    }

    // 按优先级尝试 BE/JE
    if try_be {
        log::info!("优先尝试 BE 协议: {}:{}", host, port);
        if let Ok(be) = motd_be(host, port).await {
            log::debug!("BE 状态: {}", be.status);
            if be.status == "online" {
                return Ok(fmt_be(be));
            }
        }
        log::info!("BE 失败，回退尝试 JE: {}:{}", host, port);
        if let Ok(je) = motd_je(host, port).await {
            if je.status == "online" {
                return Ok(fmt_je(je));
            }
        }
    } else {
        log::info!("优先尝试 JE 协议: {}:{}", host, port);
        if let Ok(je) = motd_je(host, port).await {
            if je.status == "online" {
                return Ok(fmt_je(je));
            }
        }
        port = port_opt.unwrap_or(19132);
        log::info!("JE 失败，回退尝试 BE: {}:{}", host, port);
        if let Ok(be) = motd_be(host, port).await {
            if be.status == "online" {
                return Ok(fmt_be(be));
            }
        }
    }

    Err("无法获取到服务器的 MOTD 信息".into())
}

/// 格式化 BE 输出
fn fmt_be(info: motd_be::MotdBEInfo) -> String {
    format!(
        "[BE] 状态: {}\nMOTD: {}\n协议版: {}\n版本: {}\n在线/最大: {}/{}\n存档名: {}\n模式: {}\n唯一ID: {}\n延迟: {}ms",
        info.status, info.motd, info.agreement, info.version,
        info.online, info.max, info.level_name, info.game_mode,
        info.server_unique_id, info.delay
    )
}

/// 格式化 JE 输出
fn fmt_je(info: motd_je::MotdJavaInfo) -> String {
    format!(
        "[JE] 状态: {}\nMOTD: {}\n协议版: {}\n版本: {}\n在线/最大: {}/{}\n在线玩家列表: {:?}\n图标: {}\n延迟: {}ms",
        info.status, info.motd, info.agreement, info.version,
        info.online, info.max, info.sample, info.favicon,
        info.delay
    )
}
