pub fn get_auth_data(access_code: &str) -> Vec<u8> {
  let config = bincode::config::standard()
    .with_little_endian()
    .with_fixed_int_encoding();

  let username = "bblp";
  let mut padded_username = [0_u8; 32];
  padded_username[..username.len()].copy_from_slice(username.as_bytes());

  let mut padded_access_code = [0_u8; 32];
  padded_access_code[..access_code.len()].copy_from_slice(access_code.as_bytes());

  let input = (
    0x40_u32,
    0x3000_u32,
    0_u32,
    0_u32,
    padded_username,
    padded_access_code,
  );

  bincode::encode_to_vec(input, config).unwrap()
}
