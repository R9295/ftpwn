// Currently 512 as there is no fixed max message size, just guessing for now.
pub const MAX_MESSAGE_SIZE: usize = 512;
pub const RDY_FOR_NEW_USERS: [u8; 3] = [50, 50, 48];
pub const USRNAME_OK: [u8; 3] = [51, 51, 49];
pub const LOGGED_IN: [u8; 3] = [50, 51, 48];
