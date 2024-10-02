use bincode::{config, encode_to_vec};

pub fn get_auth_data(access_code: &str) -> Vec<u8> {
  let config = config::standard()
    .with_little_endian()
    .with_fixed_int_encoding();

  let mut output: Vec<u8> = Vec::new();

  let username = "bblp";
  let mut padded_username = [0_u8; 32];
  padded_username[..username.len()].copy_from_slice(username.as_bytes());

  let mut padded_access_code = [0_u8; 32];
  padded_access_code[..access_code.len()].copy_from_slice(access_code.as_bytes());

  output.append(&mut encode_to_vec(0x40_u32, config).unwrap());
  output.append(&mut encode_to_vec(0x3000_u32, config).unwrap());
  output.append(&mut encode_to_vec(0_u32, config).unwrap());
  output.append(&mut encode_to_vec(0_u32, config).unwrap());
  output.append(&mut encode_to_vec(padded_username, config).unwrap());
  output.append(&mut encode_to_vec(padded_access_code, config).unwrap());

  output
}
