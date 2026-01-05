use std::{net::IpAddr, ops::Range};

use clap::Parser;

const WORKER_CHANNEL_SIZE: usize = 100;

#[derive(Parser)]
#[command(name = "uniremote-server")]
#[command(about = "Universal Remote Control Server", long_about = None)]
struct Args {
    /// Bind address specification
    /// 
    /// Examples:
    ///   --bind 192.168.1.100        Bind to IP with port autodetection
    ///   --bind :8080                Bind to localhost on port 8080
    ///   --bind 192.168.1.100:8080   Bind to IP and port
    ///   --bind :8000-8100           Bind to localhost with port range
    ///   --bind 192.168.1.100:8000-8100  Bind to IP with port range
    ///   --bind lan                  Bind to LAN IP with port autodetection (default)
    ///   --bind lan:8080             Bind to LAN IP on port 8080
    ///   --bind lan:8000-8100        Bind to LAN IP with port range
    ///   --bind [::1]:8080           Bind to IPv6 address with port (use brackets)
    #[arg(long, default_value = "lan")]
    bind: String,
}

fn parse_bind_address(bind: &str) -> anyhow::Result<uniremote_server::BindAddress> {
    const DEFAULT_PORT_RANGE: Range<u16> = 8000..8101;
    
    // Handle "lan" and "lan:..." formats
    if bind == "lan" {
        return Ok(uniremote_server::BindAddress::Lan {
            port_range: DEFAULT_PORT_RANGE,
        });
    }
    
    if let Some(port_spec) = bind.strip_prefix("lan:") {
        let port_range = parse_port_range(port_spec)?;
        return Ok(uniremote_server::BindAddress::Lan { port_range });
    }
    
    // Handle ":port" or ":port-port" (localhost)
    if let Some(port_spec) = bind.strip_prefix(':') {
        let port_range = parse_port_range(port_spec)?;
        return Ok(uniremote_server::BindAddress::Localhost { port_range });
    }
    
    // Handle IPv6 with brackets: "[::1]:port" or "[::1]:port-port"
    if bind.starts_with('[') {
        if let Some(end_bracket) = bind.find(']') {
            let ip_str = &bind[1..end_bracket];
            let ip: IpAddr = ip_str.parse()
                .map_err(|_| anyhow::anyhow!("invalid IPv6 address: {}", ip_str))?;
            
            let port_range = if end_bracket + 1 < bind.len() {
                // There's a port specification after the bracket
                if !bind[end_bracket + 1..].starts_with(':') {
                    anyhow::bail!("expected ':' after IPv6 address in brackets");
                }
                parse_port_range(&bind[end_bracket + 2..])?
            } else {
                DEFAULT_PORT_RANGE
            };
            
            return Ok(uniremote_server::BindAddress::Ip { ip, port_range });
        } else {
            anyhow::bail!("unclosed bracket in IPv6 address");
        }
    }
    
    // Handle "IP:port" or "IP:port-port" or just "IP" (for IPv4)
    if let Some((ip_str, port_spec)) = bind.rsplit_once(':') {
        // Try to parse as IP first to distinguish from IPv6 without brackets
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            // Make sure port_spec is actually a port, not part of IPv6 address
            if let Ok(port_range) = parse_port_range(port_spec) {
                return Ok(uniremote_server::BindAddress::Ip { ip, port_range });
            }
        }
    }
    
    // Just an IP address without port
    let ip: IpAddr = bind.parse()
        .map_err(|_| anyhow::anyhow!("invalid bind address: {}", bind))?;
    Ok(uniremote_server::BindAddress::Ip {
        ip,
        port_range: DEFAULT_PORT_RANGE,
    })
}

fn parse_port_range(port_spec: &str) -> anyhow::Result<Range<u16>> {
    if let Some((start_str, end_str)) = port_spec.split_once('-') {
        let start: u16 = start_str.parse()
            .map_err(|_| anyhow::anyhow!("invalid port number: {}", start_str))?;
        let end: u16 = end_str.parse()
            .map_err(|_| anyhow::anyhow!("invalid port number: {}", end_str))?;
        
        if start >= end {
            anyhow::bail!("port range start must be less than end: {}-{}", start, end);
        }
        
        Ok(start..end + 1)
    } else {
        let port: u16 = port_spec.parse()
            .map_err(|_| anyhow::anyhow!("invalid port number: {}", port_spec))?;
        Ok(port..port + 1)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let bind_addr = parse_bind_address(&args.bind)?;

    let (remotes, lua_states) = uniremote_loader::load_remotes()?;

    tracing::info!("loaded {} remotes", remotes.len());

    let (tx, rx) = tokio::sync::mpsc::channel(WORKER_CHANNEL_SIZE);
    let worker = tokio::spawn(uniremote_lua::run(rx, lua_states));

    uniremote_server::run(tx, remotes, bind_addr).await?;
    worker.await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bind_lan() {
        let result = parse_bind_address("lan").unwrap();
        assert!(matches!(result, uniremote_server::BindAddress::Lan { .. }));
    }

    #[test]
    fn test_parse_bind_lan_port() {
        let result = parse_bind_address("lan:8080").unwrap();
        if let uniremote_server::BindAddress::Lan { port_range } = result {
            assert_eq!(port_range.start, 8080);
            assert_eq!(port_range.end, 8081);
        } else {
            panic!("Expected Lan bind address");
        }
    }

    #[test]
    fn test_parse_bind_lan_port_range() {
        let result = parse_bind_address("lan:8000-8100").unwrap();
        if let uniremote_server::BindAddress::Lan { port_range } = result {
            assert_eq!(port_range.start, 8000);
            assert_eq!(port_range.end, 8101);
        } else {
            panic!("Expected Lan bind address");
        }
    }

    #[test]
    fn test_parse_bind_localhost_port() {
        let result = parse_bind_address(":8080").unwrap();
        if let uniremote_server::BindAddress::Localhost { port_range } = result {
            assert_eq!(port_range.start, 8080);
            assert_eq!(port_range.end, 8081);
        } else {
            panic!("Expected Localhost bind address");
        }
    }

    #[test]
    fn test_parse_bind_localhost_port_range() {
        let result = parse_bind_address(":8000-8100").unwrap();
        if let uniremote_server::BindAddress::Localhost { port_range } = result {
            assert_eq!(port_range.start, 8000);
            assert_eq!(port_range.end, 8101);
        } else {
            panic!("Expected Localhost bind address");
        }
    }

    #[test]
    fn test_parse_bind_ip() {
        let result = parse_bind_address("192.168.1.100").unwrap();
        if let uniremote_server::BindAddress::Ip { ip, port_range } = result {
            assert_eq!(ip.to_string(), "192.168.1.100");
            assert_eq!(port_range.start, 8000);
            assert_eq!(port_range.end, 8101);
        } else {
            panic!("Expected Ip bind address");
        }
    }

    #[test]
    fn test_parse_bind_ip_port() {
        let result = parse_bind_address("192.168.1.100:8080").unwrap();
        if let uniremote_server::BindAddress::Ip { ip, port_range } = result {
            assert_eq!(ip.to_string(), "192.168.1.100");
            assert_eq!(port_range.start, 8080);
            assert_eq!(port_range.end, 8081);
        } else {
            panic!("Expected Ip bind address");
        }
    }

    #[test]
    fn test_parse_bind_ip_port_range() {
        let result = parse_bind_address("192.168.1.100:8000-8100").unwrap();
        if let uniremote_server::BindAddress::Ip { ip, port_range } = result {
            assert_eq!(ip.to_string(), "192.168.1.100");
            assert_eq!(port_range.start, 8000);
            assert_eq!(port_range.end, 8101);
        } else {
            panic!("Expected Ip bind address");
        }
    }

    #[test]
    fn test_parse_bind_ipv6_brackets() {
        let result = parse_bind_address("[::1]").unwrap();
        if let uniremote_server::BindAddress::Ip { ip, .. } = result {
            assert_eq!(ip.to_string(), "::1");
        } else {
            panic!("Expected Ip bind address");
        }
    }

    #[test]
    fn test_parse_bind_ipv6_port() {
        let result = parse_bind_address("[::1]:8080").unwrap();
        if let uniremote_server::BindAddress::Ip { ip, port_range } = result {
            assert_eq!(ip.to_string(), "::1");
            assert_eq!(port_range.start, 8080);
            assert_eq!(port_range.end, 8081);
        } else {
            panic!("Expected Ip bind address");
        }
    }

    #[test]
    fn test_parse_port_range_invalid() {
        let result = parse_bind_address(":8100-8000");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_ip() {
        let result = parse_bind_address("999.999.999.999");
        assert!(result.is_err());
    }
}
