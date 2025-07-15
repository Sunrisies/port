use std::io::Error;
use std::net::ToSocketAddrs;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

fn scan_port(host: &str, port: u16) -> Result<(), Error> {
    let addr_str = format!("{}:{}", host, port); // 组合IP和端口
    let timeout = Duration::from_secs(1);

    // 解析带端口的完整地址
    let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?.collect();

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => {
                println!("Port {}: open", port);
                return Ok(());
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::TimedOut => continue, // 尝试下一个地址
                std::io::ErrorKind::ConnectionRefused => {
                    println!("Port {}: closed", port);
                    return Ok(());
                }
                _ => continue, // 其他错误尝试下一个地址
            },
        }
    }

    // 所有地址尝试失败
    println!("Port {}: filtered or unreachable", port);
    Ok(())
}

// scan_ports 和 main 保持不变
fn scan_ports(host: &str, start_port: u16, end_port: u16) -> Result<(), Error> {
    for port in start_port..=end_port {
        if let Err(e) = scan_port(host, port) {
            eprintln!("Error scanning port {}: {}", port, e);
        }
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
