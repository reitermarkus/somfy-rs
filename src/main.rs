use ux::u24;

use somfy::*;

fn main() {
  let rolling_code = 42;
  let remote_address = u24::new(0xFFAA11);

  let frame = Frame::builder()
    .key(0xA7)
    .command(Command::Up)
    .rolling_code(rolling_code)
    .remote_address(remote_address)
    .build();

  dbg!(frame);
}
