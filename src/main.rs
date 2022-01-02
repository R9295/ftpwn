use std::fs;
use std::path::Path;
use std::vec::Vec;
use clap::Parser;

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
    let ip_list: Vec<String> = read_file(args.ip_list);
    let passwd_list: Vec<String> = read_file(args.passwd_list);
    Ok(())
}

fn read_file(file_name: String) -> Vec<String> {
    let file_exists = Path::new(&file_name).is_file();
    if file_exists == false {
        // using as_str to type cast from String to &str as format! needs it
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let contents = fs::read_to_string(&file_name)
        .expect(format!("Error reading from file {}", file_name).as_str());
    let ip_list: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    return ip_list
}
