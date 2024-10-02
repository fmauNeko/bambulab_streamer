use std::{path::Path, pin::Pin, sync::Arc};

use auth_data::get_auth_data;
use bincode::{config, decode_from_slice};
use clap::Parser;
use log::*;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
  sync::Mutex,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// Output file, defaults to stdout
  #[arg(short, long)]
  output: Option<String>,

  /// The printer's IP address
  #[arg(short, long)]
  address: String,

  /// The printer's access code
  #[arg(short = 'c', long)]
  access_code: String,
}

mod auth_data;
mod printer_tls_client;

#[tokio::main]
async fn main() {
  stderrlog::new().module(module_path!()).init().unwrap();

  let args = Args::parse();

  let tls_client = printer_tls_client::connect(&args.address).await.unwrap();
  let tls_client = Arc::new(Mutex::new(tls_client));
  let tls_client_handle = Arc::clone(&tls_client);

  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    println!("Exiting...");
    tls_client_handle.lock().await.shutdown().await.unwrap();
  });

  tls_client
    .lock()
    .await
    .write_all(&get_auth_data(&args.access_code))
    .await
    .unwrap();

  let jpeg_start: [u8; 4] = [0xff, 0xd8, 0xff, 0xe0];
  let jpeg_end: [u8; 2] = [0xff, 0xd9];

  let mut image: Option<Vec<u8>> = None;
  let mut payload_size: usize = 0;

  loop {
    let mut buffer: [u8; 4096] = [0; 4096];
    let read_bytes = tls_client.lock().await.read(&mut buffer).await.unwrap();

    if image.is_some() && read_bytes > 0 {
      image
        .as_mut()
        .unwrap()
        .extend_from_slice(&buffer[..read_bytes]);

      let img = image.as_ref().unwrap();

      if img.len() > payload_size {
        error!(
          "Received more data than expected: {} > {}",
          image.as_ref().unwrap().len(),
          payload_size
        );

        image = None;
      } else if img.len() == payload_size {
        if img[..4] != jpeg_start {
          error!("Invalid JPEG start marker");
        } else if img[img.len() - 2..] != jpeg_end {
          error!("Invalid JPEG end marker");
        } else {
          let mut output = match &args.output {
            Some(path) => {
              let path = Path::new(path);
              Box::pin(File::create(&path).await.unwrap()) as Pin<Box<dyn AsyncWrite>>
            }
            None => Box::pin(tokio::io::stdout()) as Pin<Box<dyn AsyncWrite>>,
          };

          output.write_all(img).await.unwrap();
        }

        image = None;
      }
    } else if read_bytes == 16 {
      let config = config::standard()
        .with_little_endian()
        .with_fixed_int_encoding();

      image = Some(Vec::new());

      let mut encoded_payload_size = [0_u8; 8];
      encoded_payload_size[..4].copy_from_slice(&buffer[0..4]);
      payload_size = decode_from_slice(&encoded_payload_size, config).unwrap().0
    } else if read_bytes == 0 {
      error!("Connection rejected by the server. Check the IP address and access code.");
      break;
    }
  }
}
