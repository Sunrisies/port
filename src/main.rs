use rayon::prelude::*;
use std::io::Error;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn scan_port(host: &str, port: u16, timeout_duration: Duration) -> Result<String, Error> {
    let addr_str = format!("{}:{}", host, port);
    let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?.collect();

    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout_duration) {
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

fn scan_ports(host: &str, start_port: u16, end_port: u16, open_only: bool) -> Result<(), Error> {
    let ports: Vec<u16> = (start_port..=end_port).collect();
    let timeout = Duration::from_millis(200);
    let num_cpus = num_cpus::get();
    println!(
        "Scanning {} ports on {} using {} threads",
        ports.len(),
        host,
        num_cpus
    );
    // 创建通道用于发送结果
    let (tx, rx) = mpsc::sync_channel(num_cpus);

    // 启动顺序输出线程
    let output_thread = thread::spawn(move || {
        let mut results = std::collections::BTreeMap::new();
        let mut next_expected = start_port;

        for (port, status) in rx {
            // 存储结果
            results.insert(port, status);

            // 输出所有连续的结果
            while let Some(status) = results.get(&next_expected) {
                // 根据open_only标志决定是否输出
                if !open_only || status == "open" {
                    println!("Port {}: {}", next_expected, status);
                }

                // 无论是否输出，都从映射中移除以保持顺序
                results.remove(&next_expected);
                next_expected += 1;
            }
        }

        // 确保所有结果都被处理（理论上不需要，但安全起见）
        for (port, status) in results {
            if !open_only || status == "open" {
                println!("Port {}: {}", port, status);
            }
        }
    });

    // 使用并行迭代器扫描端口
    ports.par_iter().for_each(|&port| {
        let status = scan_port(host, port, timeout).unwrap_or_else(|e| format!("error: {}", e));

        // 发送结果到输出线程
        tx.send((port, status)).expect("Failed to send result");
    });

    // 关闭发送通道（通知输出线程结束）
    drop(tx);

    // 等待输出线程完成
    output_thread.join().expect("Output thread panicked");

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 检查是否包含 --open 标志
    let mut open_only = false;
    let mut filtered_args = Vec::new();

    for arg in &args {
        if arg == "--open" || arg == "-o" {
            open_only = true;
        } else {
            filtered_args.push(arg.clone());
        }
    }

    // 参数数量检查（过滤掉 --open 标志后）
    if filtered_args.len() < 4 {
        eprintln!(
            "Usage: {} <ip> <start_port> <end_port> [--open | -o]",
            filtered_args[0]
        );
        eprintln!("Options:");
        eprintln!("  --open, -o    Only show open ports");
        return;
    }

    let ip = &filtered_args[1];
    let start_port: u16 = filtered_args[2].parse().expect("Invalid start port");
    let end_port: u16 = filtered_args[3].parse().expect("Invalid end port");

    if let Err(e) = scan_ports(ip, start_port, end_port, open_only) {
        eprintln!("Error scanning ports: {}", e);
    }
}
