use std::fs;
use std::fs::File;
use std::path::Path;
use std::vec::Vec;

fn main() -> std::io::Result<()> {
    let ip_list: Vec<String> = get_ip_list("list.txt");
    Ok(())
}

fn get_ip_list(file_name: &str) -> Vec<String> {
    let file_exists = Path::new(file_name).is_file();
    if file_exists == false {
        // using as_str to type cast from String to &str as format! needs it
        File::create(file_name).expect(format!("Error creating file {}", file_name).as_str());
    }
    let contents = fs::read_to_string(file_name)
        .expect(format!("Error reading from file {}", file_name).as_str());
    let ip_list: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    print!("{}", contents);
    return ip_list
}
