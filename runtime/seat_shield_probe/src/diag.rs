//! SOCK_DIAG / inet_diag dump (TCP only). Harvested from ewonika `probe/src/diag.rs`.
//! No AF_PACKET. No eBPF.

use crate::inode::InodeMap;
use crate::json;
use libc::{
    bind, recv, send, sockaddr_nl, socket, AF_INET, AF_INET6, AF_NETLINK, IPPROTO_TCP,
    NETLINK_SOCK_DIAG, NLM_F_DUMP, NLM_F_REQUEST, NLMSG_DONE, NLMSG_ERROR, SOCK_CLOEXEC, SOCK_RAW,
};
use std::io::{Error, ErrorKind};
use std::mem;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::ptr;

const SOCK_DIAG_BY_FAMILY: u16 = 20;
const INET_DIAG_INFO: u8 = 2;
const NLMSG_HDRLEN: usize = 16;
const TCP_ESTABLISHED: u8 = 1;

#[derive(Debug, Clone)]
pub struct Sock {
    pub src: String,
    pub sport: u16,
    pub dst: String,
    pub dport: u16,
    pub inode: u64,
    pub pid: Option<u32>,
    pub comm: Option<String>,
    pub rtt_us: Option<u32>,
    pub retrans: Option<u32>,
}

pub fn dump_estab_tcp(inodes: &InodeMap) -> Result<Vec<Sock>, Error> {
    let mut out = Vec::new();
    for family in [AF_INET as u8, AF_INET6 as u8] {
        out.extend(dump_one(family, inodes)?);
    }
    Ok(out
        .into_iter()
        .filter(|s| !s.dst.is_empty() && s.dst != "0.0.0.0" && s.dst != "::")
        .collect())
}

fn dump_one(family: u8, inodes: &InodeMap) -> Result<Vec<Sock>, Error> {
    let fd = unsafe { socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_SOCK_DIAG) };
    if fd < 0 {
        return Err(Error::last_os_error());
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    let mut addr: sockaddr_nl = unsafe { mem::zeroed() };
    addr.nl_family = AF_NETLINK as u16;
    let rc = unsafe {
        bind(
            fd.as_raw_fd(),
            ptr::addr_of!(addr) as *const _,
            mem::size_of::<sockaddr_nl>() as u32,
        )
    };
    if rc != 0 {
        return Err(Error::last_os_error());
    }

    let mut req = [0u8; NLMSG_HDRLEN + 56];
    let total = req.len() as u32;
    req[0..4].copy_from_slice(&total.to_ne_bytes());
    req[4..6].copy_from_slice(&SOCK_DIAG_BY_FAMILY.to_ne_bytes());
    let flags = (NLM_F_REQUEST | NLM_F_DUMP) as u16;
    req[6..8].copy_from_slice(&flags.to_ne_bytes());
    req[8..12].copy_from_slice(&1u32.to_ne_bytes());
    req[12..16].copy_from_slice(&0u32.to_ne_bytes());
    req[16] = family;
    req[17] = IPPROTO_TCP as u8;
    req[18] = 1 << (INET_DIAG_INFO - 1);
    req[20..24].copy_from_slice(&0xFFFF_FFFFu32.to_ne_bytes());

    let sent = unsafe { send(fd.as_raw_fd(), req.as_ptr().cast(), req.len(), 0) };
    if sent < 0 {
        return Err(Error::last_os_error());
    }

    let mut socks = Vec::new();
    let mut buf = vec![0u8; 65_536];
    loop {
        let n = unsafe { recv(fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len(), 0) };
        if n < 0 {
            return Err(Error::last_os_error());
        }
        let n = n as usize;
        if n == 0 {
            break;
        }
        let mut off = 0usize;
        while off + NLMSG_HDRLEN <= n {
            let len = u32::from_ne_bytes(buf[off..off + 4].try_into().unwrap()) as usize;
            if len < NLMSG_HDRLEN || off + len > n {
                break;
            }
            let ntype = u16::from_ne_bytes(buf[off + 4..off + 6].try_into().unwrap());
            if ntype == NLMSG_DONE as u16 {
                return Ok(socks);
            }
            if ntype == NLMSG_ERROR as u16 {
                return Err(Error::new(ErrorKind::Other, "inet_diag NLMSG_ERROR"));
            }
            if ntype == SOCK_DIAG_BY_FAMILY {
                if let Some(sock) = parse_diag(&buf[off + NLMSG_HDRLEN..off + len], inodes) {
                    socks.push(sock);
                }
            }
            off += (len + 3) & !3;
        }
    }
    Ok(socks)
}

fn parse_diag(payload: &[u8], inodes: &InodeMap) -> Option<Sock> {
    if payload.len() < 72 {
        return None;
    }
    let family = payload[0];
    let state = payload[1];
    if state != TCP_ESTABLISHED {
        return None;
    }
    let sport = u16::from_be_bytes(payload[4..6].try_into().ok()?);
    let dport = u16::from_be_bytes(payload[6..8].try_into().ok()?);
    let src = parse_ip(family, &payload[8..24])?;
    let dst = parse_ip(family, &payload[24..40])?;
    let inode = u32::from_ne_bytes(payload[68..72].try_into().ok()?) as u64;
    let (pid, comm) = inodes.get(&inode).cloned().map_or((None, None), |(p, c)| {
        (Some(p), Some(c))
    });
    let mut rtt = None;
    let mut retrans = None;
    if let Some(info) = find_nla(payload, 72, INET_DIAG_INFO as u16) {
        if info.len() >= 8 + 24 * 4 {
            let base = 8;
            let u32_at = |i: usize| {
                u32::from_ne_bytes(info[base + i * 4..base + i * 4 + 4].try_into().unwrap())
            };
            retrans = Some(u32_at(23));
            rtt = Some(u32_at(15));
        }
    }
    Some(Sock {
        src,
        sport,
        dst,
        dport,
        inode,
        pid,
        comm,
        rtt_us: rtt,
        retrans,
    })
}

fn find_nla(buf: &[u8], mut off: usize, want: u16) -> Option<&[u8]> {
    while off + 4 <= buf.len() {
        let nla_len = u16::from_ne_bytes(buf[off..off + 2].try_into().ok()?) as usize;
        let nla_type = u16::from_ne_bytes(buf[off + 2..off + 4].try_into().ok()?);
        if nla_len < 4 || off + nla_len > buf.len() {
            break;
        }
        if nla_type == want {
            return Some(&buf[off + 4..off + nla_len]);
        }
        off += (nla_len + 3) & !3;
    }
    None
}

fn parse_ip(family: u8, raw: &[u8]) -> Option<String> {
    if family == AF_INET as u8 {
        if raw.len() < 4 {
            return None;
        }
        Some(Ipv4Addr::new(raw[0], raw[1], raw[2], raw[3]).to_string())
    } else if family == AF_INET6 as u8 {
        if raw.len() < 16 {
            return None;
        }
        let mut oct = [0u8; 16];
        oct.copy_from_slice(&raw[..16]);
        let ip = Ipv6Addr::from(oct);
        if let Some(v4) = ip.to_ipv4_mapped() {
            Some(v4.to_string())
        } else {
            Some(ip.to_string())
        }
    } else {
        None
    }
}

pub fn sock_json(s: &Sock) -> String {
    format!(
        "{{\"proto\":\"tcp\",\"state\":\"ESTAB\",\"src\":{},\"sport\":{},\"dst\":{},\"dport\":{},\"inode\":{},\"pid\":{},\"comm\":{},\"rtt_us\":{},\"retrans\":{}}}",
        json::escape(&s.src),
        s.sport,
        json::escape(&s.dst),
        s.dport,
        s.inode,
        json::opt_u32(s.pid),
        json::opt_str(&s.comm),
        json::opt_u32(s.rtt_us),
        json::opt_u32(s.retrans),
    )
}
