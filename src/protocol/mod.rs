pub enum Command {
    Response(bool),
    Authenticate { digest: [u8; 32] },
}

impl Command {
    // pub fn Re
}
