use std::fs;
use std::path::Path;
use std::vec::Vec;
use std::env;

fn main() -> std::io::Result<()> {
    let ip_list: Vec<String> = read_file("list.txt");
    let passwd_list: Vec<String> = read_file("passwd.txt");
    let args: Vec<String> = env::args().collect();
    println!("{:?}", ip_list);
    println!("{:?}", args);
    Ok(())
}

fn read_file(file_name: &str) -> Vec<String> {
    let file_exists = Path::new(file_name).is_file();
    if file_exists == false {
        // using as_str to type cast from String to &str as format! needs it
        panic!("{}", format!("File {} does not exist!", file_name))
    }
    let contents = fs::read_to_string(file_name)
        .expect(format!("Error reading from file {}", file_name).as_str());
    let ip_list: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    print!("{}", contents);
    return ip_list
}
