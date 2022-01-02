use std::option::Option;
/*
    Address is the class used to encapsulate IPs
*/

#[derive(Default)]
pub struct Address {
    ip: String,   // IP Address
    attempts: u8, // Attempts // TODO: will go out of bounds if there are more than 255 passwords
    is_successful: bool,
    successful_password: String,
}

impl Address {
    pub fn new(ip: &String) -> Self {
        Self {
            ip: ip.clone(),
            attempts: 0,
            is_successful: false,
            successful_password: String::from(""),
        }
    }
    pub fn attempt(&mut self, password: &String) {
        self.attempts += 1
    }
    pub fn successful_password(&self) -> Option<&String> {
        if !self.is_successful {
            return None;
        }
        return Some(&self.successful_password);
    }
    pub fn is_successful(&self) -> bool {
        return self.is_successful;
    }
}
