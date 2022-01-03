use clap::Parser;
use std::fs;
use std::path::Path;
use std::vec::Vec;
mod address;
use address::Address;
use std::io::Read;
use std::net::TcpStream;
mod constants;
use constants::MAX_MESSAGE_SIZE;

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
    let mut addresses = Vec::new();
    // TODO make into a generator (generators are not stable yet)
    for host in &host_list {
        addresses.push(Address::new(host))
    }
    println!("Starting...");
    for addr in &mut addresses {
        let mut stream = TcpStream::connect(addr.get_host())?;
        let mut buffer = [0; MAX_MESSAGE_SIZE];
        stream.read(&mut buffer)?;
        // clear buffer with welcome message
        drop(buffer);
        for passwd in &credential_list {
            let iter: Vec<&str> = passwd.split(":").collect();
            addr.attempt(&iter[0], &iter[1], &mut stream)?;
            if addr.is_successful() == true {
                println!(
                    "Success on {:?} with credentials {:?}. Total attempts: {:?}",
                    addr.get_host(),
                    addr.get_successful_password().unwrap(),
                    addr.attempts()
                );
                break;
            }
        }
    }
    Ok(())
}

fn read_file_lines(file_name: &String) -> Vec<String> {
    let file_exists = Path::new(&file_name).is_file();
    if file_exists == false {
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let contents = fs::read_to_string(&file_name)
        .expect(format!("Error reading from file {}", file_name).as_str());
    let lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    return lines;
}
