use std::env;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = env::args().collect();
    let port: u16 = args.get(1).and_then(|p| p.parse().ok()).unwrap_or(8090);
    let duration_secs: u64 = args.get(2).and_then(|d| d.parse().ok()).unwrap_or(300);

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to 127.0.0.1:{}: {}", port, e);
            std::process::exit(1);
        }
    };

    println!("Direct test listener active on 127.0.0.1:{} for {}s", port, duration_secs);

    let _ = listener.set_nonblocking(true);
    let start = Instant::now();

    while start.elapsed() < Duration::from_secs(duration_secs) {
        if let Ok((_stream, _)) = listener.accept() {
            // Drop connection immediately
        }
        thread::sleep(Duration::from_millis(50));
    }
}
