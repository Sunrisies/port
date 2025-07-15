use rayon::prelude::*;
use std::collections::BTreeMap;
use std::io::Error;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use std::time::Duration;

fn scan_port(host: &str, port: u16) -> Result<String, Error> {
    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_millis(200); // 更短的超时时间
    let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?.collect();

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => return Ok("open".to_string()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::TimedOut => continue,
                std::io::ErrorKind::ConnectionRefused => return Ok("closed".to_string()),
                _ => continue,
            },
        }
    }

    Ok("filtered or unreachable".to_string())
}

fn scan_ports(host: &str, start_port: u16, end_port: u16) -> Result<(), Error> {
    let ports: Vec<u16> = (start_port..=end_port).collect();

    // 使用线程安全的BTreeMap收集结果（自动按键排序）
    let results = Mutex::new(BTreeMap::new());

    ports.par_iter().for_each(|&port| {
        let status = scan_port(host, port).unwrap_or_else(|e| format!("error: {}", e));
        let mut map = results.lock().unwrap();
        map.insert(port, status);
    });

    // 按端口顺序输出结果
    let map = results.lock().unwrap();
    for (port, status) in map.iter() {
        println!("Port {}: {}", port, status);
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <ip> <start_port> <end_port>", args[0]);
        return;
    }
    let ip = &args[1];
    let start_port: u16 = args[2].parse().expect("Invalid start port");
    let end_port: u16 = args[3].parse().expect("Invalid end port");

    if let Err(e) = scan_ports(ip, start_port, end_port) {
        eprintln!("Error scanning ports: {}", e);
    }
}
