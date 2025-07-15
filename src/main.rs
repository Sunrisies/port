use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::Error;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::time::Duration;

fn scan_port(host: &str, port: u16) -> Result<String, Error> {
    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_millis(200);
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
    let (tx, rx) = mpsc::channel();

    // 使用并行迭代器发送结果
    ports.par_iter().for_each_with(tx.clone(), |tx, &port| {
        let status = scan_port(host, port).unwrap_or_else(|e| format!("error: {}", e));
        tx.send((port, status)).expect("Failed to send result");
    });

    // 释放发送端，这样接收端知道何时结束
    drop(tx);

    // 使用最小堆管理结果
    let mut heap = BinaryHeap::new();
    let mut next_expected = start_port;

    for (port, status) in rx {
        // 如果收到的是下一个期望的端口，直接输出
        if port == next_expected {
            println!("Port {}: {}", port, status);
            next_expected += 1;

            // 检查堆中是否有连续的端口可以输出
            while let Some(Reverse((min_port, min_status))) = heap.peek() {
                if *min_port == next_expected {
                    println!("Port {}: {}", min_port, min_status);
                    next_expected += 1;
                    heap.pop();
                } else {
                    break;
                }
            }
        } else {
            // 不是下一个期望的端口，加入最小堆
            heap.push(Reverse((port, status)));
        }
    }

    // 输出堆中剩余的结果（按顺序）
    while let Some(Reverse((port, status))) = heap.pop() {
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
