#[derive(Debug)]
pub struct MyxSerialReceiver<const L: usize, const ID: u8> {
    storage: [u8; L],
    received: usize,
    length: usize,
    is_correct: bool,
    has_ticks: bool,
}

impl<const L: usize, const ID: u8> MyxSerialReceiver<L, ID> {
    const L_DATA: usize = L - 3;

    pub fn new() -> Self {
        if L < 3 {
            panic!("MyxSerialReceiver should use at least 3 bytes of storage");
        }
        Self {
            storage: [0; L],
            received: 0,
            length: 0,
            is_correct: false,
            has_ticks: false,
        }
    }

    pub fn receive<'a>(&'a mut self, data: u8) -> MyxSerialReceiveState<'a> {
        self.has_ticks = false;
        match self.received {
            0 => {
                self.storage[self.received] = data;
                self.received = 1;
                if data == ID {
                    self.is_correct = true;
                    MyxSerialReceiveState::Incomplete
                } else {
                    self.is_correct = false;
                    MyxSerialReceiveState::ErrorID
                }
            }
            1 => {
                self.storage[self.received] = data;
                self.received = 2;
                self.length = data as usize;
                if self.is_correct {
                    if self.length > Self::L_DATA {
                        self.is_correct = false;
                        MyxSerialReceiveState::ErrorLength
                    } else {
                        MyxSerialReceiveState::Incomplete
                    }
                } else {
                    MyxSerialReceiveState::Incomplete
                }
            }
            _ => {
                if self.is_correct {
                    self.storage[self.received] = data;
                    self.received += 1;
                    if self.received == self.length + 3 {
                        self.received = 0;
                        let checksum = checksum(&self.storage[0..self.length + 2]);
                        if checksum == self.storage[self.length + 2] {
                            MyxSerialReceiveState::Complete(&self.storage[2..self.length + 2])
                        } else {
                            MyxSerialReceiveState::ErrorChecksum
                        }
                    } else {
                        MyxSerialReceiveState::Incomplete
                    }
                } else {
                    self.received += 1;
                    if self.received == self.length + 3 {
                        self.received = 0;
                    }
                    MyxSerialReceiveState::Incomplete
                }
            }
        }
    }

    pub fn on_tick<'a>(&'a mut self) -> MyxSerialReceiveState<'a> {
        if self.received != 0 {
            if !self.has_ticks {
                self.has_ticks = true;
                MyxSerialReceiveState::Incomplete
            } else {
                self.received = 0;
                MyxSerialReceiveState::ErrorTimeout
            }
        } else {
            MyxSerialReceiveState::Incomplete
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MyxSerialReceiveState<'a> {
    ErrorTimeout,
    ErrorID,
    ErrorLength,
    ErrorChecksum,
    Incomplete,
    Complete(&'a [u8]),
}

pub fn checksum(data: &[u8]) -> u8 {
    let mut result: u16 = 0;
    for i in 0..data.len() {
        result += data[i] as u16;
    }
    result = (result >> 8) + (result & 0xff);
    result = (result >> 8) + (result & 0xff);
    (result & 0xff) as u8
}

pub fn checksum_id_len(id: u8, len: u8, data: &[u8]) -> u8 {
    let mut result: u16 = 0;
    result += id as u16;
    result += len as u16;
    for i in 0..data.len() {
        result += data[i] as u16;
    }
    result = (result >> 8) + (result & 0xff);
    result = (result >> 8) + (result & 0xff);
    (result & 0xff) as u8
}

pub fn send(id: u8, data: &[u8], mut sender: impl FnMut(u8)) {
    if data.len() > 0xff {
        panic!("Data length should be less than 256");
    }
    sender(id);
    sender(data.len() as u8);
    for i in 0..data.len() {
        sender(data[i]);
    }
    sender(checksum_id_len(id, data.len() as u8, data));
}
