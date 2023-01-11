use crate::{send, MyxSerialReceiveState, MyxSerialReceiver};
use rand::{Rng, SeedableRng};

#[test]
fn test_normal() {
    const ID: u8 = 0x03;
    const LEN: usize = 32;
    const NUM_TESTS: usize = 128;

    let mut receiver = MyxSerialReceiver::<{ LEN + 3 }, ID>::new();
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    for _ in 0..NUM_TESTS {
        let len = rng.gen_range(0..LEN as usize);
        let mut data = vec![0u8; len];
        rng.fill(&mut data[..]);

        let mut received_value: Option<Box<[u8]>> = None;

        send(ID, &data[..], |b| {
            let result = receiver.receive(b);
            match result {
                MyxSerialReceiveState::Incomplete => {
                    if received_value.is_some() {
                        panic!("Received more.");
                    }
                }
                MyxSerialReceiveState::Complete(values) => {
                    received_value = Some(values.into());
                }
                _ => {
                    let result = format!("{:?}", result);
                    panic!(
                        "Receive Error: {:?}. data: {:?}, receiver: {:?}",
                        result, data, receiver
                    );
                }
            }
        });
        assert!(received_value.is_some());
        assert_eq!(received_value.unwrap(), data.into());
    }
}
