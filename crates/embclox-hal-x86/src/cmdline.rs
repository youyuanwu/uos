//! Kernel command-line parsing for boot-time network mode selection.
//!
//! Bootloader-agnostic: takes a `&str` (caller fetches from Limine,
//! Multiboot, or wherever) and returns a structured [`NetMode`].
//!
//! Recognised tokens (whitespace-separated):
//! - `net=dhcp` → [`NetMode::Dhcp`]
//! - `net=static` → [`NetMode::Static`] using `default_static`
//! - `ip=A.B.C.D/N` → override static IPv4 + prefix
//! - `gw=A.B.C.D` → override static gateway
//!
//! Unknown tokens are ignored. An empty cmdline parses as
//! `NetMode::Static` with the caller-provided defaults.

/// Network configuration mode selected by the kernel cmdline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    Dhcp,
    Static {
        ip: [u8; 4],
        prefix: u8,
        gw: [u8; 4],
    },
}

/// Default static configuration for an example.
#[derive(Debug, Clone, Copy)]
pub struct StaticDefaults {
    pub ip: [u8; 4],
    pub prefix: u8,
    pub gw: [u8; 4],
}

/// Parse the `net=...`, `ip=A.B.C.D/N`, and `gw=A.B.C.D` tokens from a
/// whitespace-separated kernel command line.
///
/// `defaults` supplies the static IP, prefix, and gateway used when the
/// cmdline doesn't override them (or selects `net=static` without
/// specifying `ip=`/`gw=`).
pub fn parse_net_mode(cmdline: &str, defaults: StaticDefaults) -> NetMode {
    let mut mode = "static";
    let mut ip_cidr: Option<&str> = None;
    let mut gw: Option<&str> = None;
    for tok in cmdline.split_ascii_whitespace() {
        if let Some(rest) = tok.strip_prefix("net=") {
            mode = rest;
        } else if let Some(rest) = tok.strip_prefix("ip=") {
            ip_cidr = Some(rest);
        } else if let Some(rest) = tok.strip_prefix("gw=") {
            gw = Some(rest);
        }
    }

    if mode.eq_ignore_ascii_case("dhcp") {
        return NetMode::Dhcp;
    }

    let (ip, prefix) = ip_cidr
        .and_then(parse_ipv4_cidr)
        .unwrap_or((defaults.ip, defaults.prefix));
    let gw = gw.and_then(parse_ipv4).unwrap_or(defaults.gw);
    NetMode::Static { ip, prefix, gw }
}

/// Parse `A.B.C.D` into octets.
pub fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let mut parts = s.split('.');
    let a = parts.next()?.parse::<u8>().ok()?;
    let b = parts.next()?.parse::<u8>().ok()?;
    let c = parts.next()?.parse::<u8>().ok()?;
    let d = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([a, b, c, d])
}

/// Parse `A.B.C.D/PREFIX` into octets and prefix length (0..=32).
pub fn parse_ipv4_cidr(s: &str) -> Option<([u8; 4], u8)> {
    let (ip_part, prefix_part) = s.split_once('/')?;
    let ip = parse_ipv4(ip_part)?;
    let prefix = prefix_part.parse::<u8>().ok()?;
    if prefix > 32 {
        return None;
    }
    Some((ip, prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: StaticDefaults = StaticDefaults {
        ip: [10, 0, 0, 5],
        prefix: 24,
        gw: [10, 0, 0, 1],
    };

    #[test]
    fn empty_cmdline_uses_defaults() {
        assert_eq!(
            parse_net_mode("", D),
            NetMode::Static {
                ip: [10, 0, 0, 5],
                prefix: 24,
                gw: [10, 0, 0, 1]
            }
        );
    }

    #[test]
    fn net_dhcp() {
        assert_eq!(parse_net_mode("net=dhcp", D), NetMode::Dhcp);
        assert_eq!(parse_net_mode("foo net=DHCP bar", D), NetMode::Dhcp);
    }

    #[test]
    fn net_static_with_overrides() {
        assert_eq!(
            parse_net_mode("net=static ip=192.168.1.50/16 gw=192.168.1.1", D),
            NetMode::Static {
                ip: [192, 168, 1, 50],
                prefix: 16,
                gw: [192, 168, 1, 1]
            }
        );
    }

    #[test]
    fn unknown_tokens_ignored() {
        assert_eq!(
            parse_net_mode("foo=bar baz net=dhcp quux", D),
            NetMode::Dhcp
        );
    }

    #[test]
    fn malformed_ip_falls_back_to_default() {
        assert_eq!(
            parse_net_mode("net=static ip=not.an.ip/24", D),
            NetMode::Static {
                ip: [10, 0, 0, 5],
                prefix: 24,
                gw: [10, 0, 0, 1]
            }
        );
    }

    #[test]
    fn out_of_range_prefix_falls_back() {
        assert_eq!(parse_ipv4_cidr("10.0.0.1/33"), None);
    }
}
