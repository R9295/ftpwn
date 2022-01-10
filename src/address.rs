/*
    Address is the class used to encapsulate hosts
*/

use crate::constants::MAX_MESSAGE_SIZE;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::option::Option;
use std::str;

#[derive(Default)]
pub struct Address {
    host: String, // Host. eg: something.something
    attempts: u8, // Attempts // TODO: will go out of bounds if there are more than 255 passwords
    is_successful: bool,
    successful_credentials: String,
}

impl Address {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
            attempts: 0,
            is_successful: false,
            successful_credentials: String::from(""),
        }
    }
    pub fn check_no_auth(&mut self) {
        // TODO Checks if the FTP server allows anon
        self.attempts += 1
    }
    pub fn attempt(
        &mut self,
        user: &str,
        password: &str,
        stream: &mut TcpStream,
    ) -> std::io::Result<()> {
        if self.attempts == 0 {
            self.check_no_auth()
        }
        stream.write(format!("USER {}\r\n", user).as_bytes())?;
        let mut buffer = [0; MAX_MESSAGE_SIZE];
        stream.read(&mut buffer)?;
        // code 331
        if buffer[..3] == [51, 51, 49] {
            stream.write(format!("PASS {}\r\n", password).as_bytes())?;
            stream.read(&mut buffer)?;
            // code 230
            if buffer[..3] == [50, 51, 48] {
                self.successful_credentials = format!("{}:{}", user, password);
                self.is_successful = true
            }
        }

        self.attempts += 1;
        Ok(())
    }
    pub fn get_successful_credentials(&self) -> Option<&str> {
        if !self.is_successful {
            return None;
        }
        Some(&self.successful_credentials)
    }
    pub fn is_successful(&self) -> bool {
        self.is_successful
    }
    pub fn attempts(&self) -> u8 {
        self.attempts
    }
    pub fn get_host(&self) -> &str {
        &self.host
    }
}
