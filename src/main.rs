use rayon::prelude::*;
use std::io::Error;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration; // 引入并行处理库

fn scan_port(host: &str, port: u16) -> Result<(), Error> {
    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_millis(500); // 更短的超时时间
    let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?.collect();

    let mut result = None;

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => {
                result = Some(("open", port));
                break;
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::TimedOut => {
                    // 继续尝试下一个地址
                }
                std::io::ErrorKind::ConnectionRefused => {
                    result = Some(("closed", port));
                    break;
                }
                _ => {
                    // 其他错误暂时忽略
                }
            },
        }
    }

    // 输出结果
    match result {
        Some(("open", p)) => println!("Port {}: open", p),
        Some(("closed", p)) => println!("Port {}: closed", p),
        _ => println!("Port {}: filtered or unreachable", port),
    }

    Ok(())
}

fn scan_ports(host: &str, start_port: u16, end_port: u16) -> Result<(), Error> {
    // 创建端口范围集合
    let ports: Vec<u16> = (start_port..=end_port).collect();

    // 使用并行迭代处理所有端口
    ports.par_iter().for_each(|&port| {
        if let Err(e) = scan_port(host, port) {
            eprintln!("Error scanning port {}: {}", port, e);
        }
    });

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
