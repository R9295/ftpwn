use std::option::Option;
/*
    Address is the class used to encapsulate hosts
*/

#[derive(Default)]
pub struct Address {
    host: String, // Host. eg: ftp://something.something
    attempts: u8, // Attempts // TODO: will go out of bounds if there are more than 255 passwords
    is_successful: bool,
    successful_password: String,
}

impl Address {
    pub fn new(host: &String) -> Self {
        Self {
            host: host.clone(),
            attempts: 0,
            is_successful: false,
            successful_password: String::from(""),
        }
    }
    pub fn check_no_auth(&mut self) {
        // Checks if the FTP server requires authentication
        self.attempts += 1
    }
    pub fn attempt(&mut self, password: &String) {
        if self.attempts == 0 {
            self.check_no_auth()
        }
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
    pub fn get_host(&self) -> &String {
        return &self.host;
    }
}
