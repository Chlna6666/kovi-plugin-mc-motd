use std::net::IpAddr;
use regex::Regex;
use lazy_static::lazy_static;

/// 检查 IP 地址或域名是否合法
pub fn is_valid_ip_or_domain(host: &str) -> bool {
    // 只保留主机部分，不含端口
    let host = host.split(':').next().unwrap_or(host);

    // 尝试解析为 IP 地址
    if host.parse::<IpAddr>().is_ok() {
        return true;
    }

    // 使用正则检查域名格式
    // 支持子域名和顶级域名，标签以字母/数字开头和结尾，中间可含中划线
    lazy_static! {
        static ref DOMAIN_REGEX: Regex = Regex::new(
            r"^(?xi)
            [a-z0-9]                                    # 起始字符
            (?:[a-z0-9-]{0,61}[a-z0-9])?                 # 可选中间字符
            (?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)* # 子域名部分
            $"
        ).unwrap();
    }

    DOMAIN_REGEX.is_match(host)
}

/// 检查端口号是否合法（1-65535）
pub fn is_valid_port(port: u16) -> bool {
    (1..=65535).contains(&port)
}
