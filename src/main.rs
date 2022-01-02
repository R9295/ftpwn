use clap::Parser;
use std::fs;
use std::path::Path;
use std::vec::Vec;
mod address;
use address::Address;

#[derive(Parser, Debug)]
struct Args {
    // .txt file containing list of IPs
    #[clap(short, long)]
    ip_list: String,
    // .txt file containing list of passwords
    #[clap(short, long)]
    passwd_list: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let ip_list: Vec<String> = read_file(&args.ip_list);
    println!("Found {:?} IPs", ip_list.len());
    let passwd_list: Vec<String> = read_file(&args.passwd_list);
    println!("Found {:?} passwords", passwd_list.len());
    let mut addresses = Vec::new();
    // TODO make into a generator (generators are not stable yet)
    for ip in &ip_list {
        addresses.push(Address::new(ip))
    }
    Ok(())
}

fn read_file(file_name: &String) -> Vec<String> {
    let file_exists = Path::new(&file_name).is_file();
    if file_exists == false {
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let contents = fs::read_to_string(&file_name)
        .expect(format!("Error reading from file {}", file_name).as_str());
    let ip_list: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    return ip_list;
}
