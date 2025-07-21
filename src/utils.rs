use std::net::{IpAddr, ToSocketAddrs};

// 检查 IP 地址或域名是否合法
pub fn is_valid_ip_or_domain(host: &str) -> bool {
    // 分割 host 和端口（如果有端口）
    let host = host.split(':').next().unwrap_or(host); // 获取主机部分
    
    // 先尝试解析为 IP 地址，若解析失败则检查域名
    if let Ok(_) = host.parse::<IpAddr>() {
        return true; // 如果是有效的 IP 地址，直接返回 true
    }

    // 尝试进行域名解析
    let addr = format!("{}:0", host); // 使用端口占位符
    addr.to_socket_addrs()
        .map(|mut addrs| addrs.next().is_some()) // 如果域名解析成功，则返回 true
        .unwrap_or(false)
}

// 检查端口号是否合法（1-65535）
pub fn is_valid_port(port: u16) -> bool {
    port >= 1 && port <= 65535
}
