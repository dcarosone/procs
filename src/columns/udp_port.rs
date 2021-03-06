use crate::Column;
use procfs::{FDTarget, Io, ProcResult, Process, UdpNetEntry};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct UdpPort {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
    udp_entry: Vec<UdpNetEntry>,
    udp6_entry: Vec<UdpNetEntry>,
}

impl UdpPort {
    pub fn new() -> Self {
        let header = String::from("UDP");
        let unit = String::from("");
        UdpPort {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
            udp_entry: procfs::udp().unwrap(),
            udp6_entry: procfs::udp6().unwrap(),
        }
    }
}

impl Column for UdpPort {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let mut socks = Vec::new();
        if let Ok(fds) = curr_proc.fd() {
            for fd in fds {
                if let FDTarget::Socket(x) = fd.target {
                    socks.push(x)
                }
            }
        }
        let mut ports = Vec::new();
        for sock in &socks {
            let mut udp_iter = self.udp_entry.iter().chain(self.udp6_entry.iter());
            let entry = udp_iter.find(|&x| x.inode == *sock);
            if let Some(entry) = entry {
                ports.push(entry.local_address.port());
            }
        }
        ports.sort();
        ports.dedup();
        let fmt_content = format!("{:?}", ports);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    fn display_header(&self) -> String {
        crate::util::expand(&self.header, self.max_width)
    }

    fn display_unit(&self) -> String {
        crate::util::expand(&self.unit, self.max_width)
    }

    fn display_content(&self, pid: i32) -> Option<String> {
        if let Some(content) = self.fmt_contents.get(&pid) {
            Some(crate::util::expand(content, self.max_width))
        } else {
            None
        }
    }

    fn find_partial(&self, pid: i32, keyword: &str) -> bool {
        if let Some(content) = self.fmt_contents.get(&pid) {
            content.find(keyword).is_some()
        } else {
            false
        }
    }

    fn find_exact(&self, pid: i32, keyword: &str) -> bool {
        if let Some(content) = self.fmt_contents.get(&pid) {
            let content = content.replace("[", "").replace("]", "");
            let content = content.split(',');
            for c in content {
                if c == keyword {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    fn sorted_pid(&self, order: &crate::config::ConfigSortOrder) -> Vec<i32> {
        let mut contents: Vec<(&i32, &String)> = self.raw_contents.iter().collect();
        contents.sort_by_key(|&(_x, y)| y);
        if let crate::config::ConfigSortOrder::Descending = order {
            contents.reverse()
        }
        contents.iter().map(|(x, _y)| **x).collect()
    }

    fn reset_max_width(&mut self) {
        self.max_width = std::cmp::max(self.header.len(), self.unit.len());
    }

    fn update_max_width(&mut self, pid: i32) {
        if let Some(content) = self.fmt_contents.get(&pid) {
            self.max_width = cmp::max(content.len(), self.max_width);
        }
    }
}
