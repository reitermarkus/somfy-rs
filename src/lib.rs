use std::vec::Vec;

enum ControlCode {
  My      = 0x1,
	Up      = 0x2,
	MyUp    = 0x3,
	Down    = 0x4,
	MyDown  = 0x5,
	UpDown  = 0x6,
	Prog    = 0x8,
	SunFlag = 0x9,
	Flag    = 0xA
}

#[derive(Debug)]
struct Frame {
  payload: Vec<u8>
}

impl Frame {
  fn new(control_code: ControlCode, rolling_code: u16, address: u32) -> Frame {
    let rolling_code_bytes = rolling_code.to_be_bytes();
    let address_bytes = address.to_le_bytes();

    let mut payload: Vec<u8> = vec![0; 7];
    payload[0] = 0xA0;
    payload[1] = (control_code as u8) << 4;
    payload[2] = rolling_code_bytes[1];
    payload[3] = rolling_code_bytes[0];
    payload[4] = address_bytes[2];
    payload[5] = address_bytes[1];
    payload[6] = address_bytes[0];

    let mut checksum: u8 = 0;
    for byte in &payload {
      checksum = checksum ^ byte ^ (byte >> 4)
    }
    checksum &= 0b1111;

    payload[1] |= checksum;

    Frame {
      payload
    }
  }
}

#[test]
fn test_frame() {
  let frame = Frame::new(ControlCode::My, 0, 0);
  for byte in frame.payload {
    println!("{:08b}", byte)
  }
}
