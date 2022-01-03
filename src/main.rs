use clap::Parser;
use std::fs;
use std::path::Path;
use std::vec::Vec;
mod address;
use address::Address;
use std::net::TcpStream;
use std::error::Error;

#[derive(Parser, Debug)]
struct Args {
    // .txt file containing list of IPs
    #[clap(short, long)]
    host_list: String,
    // .txt file containing list of passwords
    #[clap(short, long)]
    passwd_list: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let host_list: Vec<String> = read_file_lines(&args.host_list);
    println!("Found {:?} Hosts", host_list.len());
    let passwd_list: Vec<String> = read_file_lines(&args.passwd_list);
    println!("Found {:?} passwords", passwd_list.len());
    let mut addresses = Vec::new();
    // TODO make into a generator (generators are not stable yet)
    for host in &host_list {
        addresses.push(Address::new(host))
    }
    println!("Starting...");
    for addr in &addresses {
        let stream = match TcpStream::connect(addr.get_host()) {
            Ok(stream) => {
                println!("Successfully connected to {:?}", addr.get_host());
            },
            Err(err) => {
                eprintln!("Could not connect to {:?}, Error: {:?}", addr.get_host(), err);
            }
        };
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
