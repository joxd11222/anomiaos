pub Struct MemoryBlock {
    start_address: usize,
    size: usize,
    data: [u8; 1024],
    used: bool,
}

impl MemoryBlock {
    fn new() -> Self {
        Self {
            start_address: 0,
            size: 0,
            data: [0; 1024],
            used: false,
        }
    }
}

