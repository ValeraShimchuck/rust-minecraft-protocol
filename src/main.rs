use std::cell::Cell;
use std::io::Read;
use std::io::{Error, Result, ErrorKind};
use bytebuffer::{ByteBuffer, ByteReader};

use tokio::io::{self, AsyncReadExt};
use tokio::net::TcpListener;


const SEG_BITS: u32 = 0x7F;
const CON_BIT: u32 = 0x80;

//struct ByteBuffer {
//    index: Cell<usize>,
//    buffer: Vec<u8>
//}

trait MinecraftReadTypes {
    fn read_var_int(&mut self) -> std::io::Result<u32>;

    
    fn read_var_string(&mut self) -> std::io::Result<String>;
}

trait MinecraftWriteTypes {
    
    fn write_var_int(&mut self, int: u32) -> Result<()>;

    fn write_var_string(&mut self, str: &str) -> Result<()>;

}

impl<'a> MinecraftReadTypes for ByteReader<'a> {


    fn read_var_int(&mut self) -> Result<u32> {
        let mut value: u32 = 0;
        let mut position: u8 = 0;
        let mut currentByte: u8;
        loop {
            currentByte = self.read_u8()?;
            value |= (currentByte as u32 & SEG_BITS) << position;
            if (currentByte as u32 & CON_BIT) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                return Err(Error::new(ErrorKind::InvalidData, "Var int is more then 5 byte long"));
            }
        }
        return Ok(value);
    }

    fn read_var_string(&mut self) -> Result<String> {
        let size = self.read_var_int()?;
        match String::from_utf8(self.read_bytes(size as usize)?) {
            Ok(string_result) => Ok(string_result),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }
    


}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("starting server");
    let listener = TcpListener::bind("127.0.0.1:25565").await?;
    println!("started server");
    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("accepted connection");
        let mut state: u8 = 0;
        tokio::spawn(async move {
            let mut buffer = vec![0;2048];
            loop {
                println!("prepare reading");
                let n = socket.read(&mut buffer).await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }
                println!("got a packet {n}");
                let mut read_buffer: ByteReader = ByteReader::from_bytes(&buffer);
                let packet_length = read_buffer.read_var_int().unwrap();
                let packet_id = read_buffer.read_var_int().unwrap();
                println!("packet info {n} {packet_length} id: {packet_id}");
                
                if true {
                    continue;
                }
                if packet_id == 0 && state == 0 {
                    let protocol_version = read_buffer.read_var_int().unwrap();
                    let server_address = read_buffer.read_var_string().unwrap();
                    let server_port = read_buffer.read_u16().unwrap();
                    let game_state = read_buffer.read_var_int().unwrap();
                    state = game_state as u8;
                    println!("Got handshake protocol: {protocol_version} address: {server_address} port: {server_port} state: {game_state}");
                }
                if packet_id == 0 && state == 1 {
                    // there is not fields in this packet, so sending a packet
                }
            }
        });
    }
}

//fn readVarInt(bytes: &[u8]) -> Result<(u32, usize), ()> {
//    let mut value: u32 = 0;
//    let mut position: u8 = 0;
//    let mut index: u8 = 0;
//    let mut currentByte: u8;
//    loop {
//        currentByte = bytes[index as usize];
//        index += 1;
//        value |= (currentByte as u32 & SEG_BITS) << position;
//        if (currentByte as u32 & CON_BIT) == 0 {
//            break;
//        }
//        position += 7;
//        if position >= 32 {
//            return Err(());
//        }
//    }
//    return Ok((value, index as usize));
//}
//
//fn readString(bytes: &[u8]) -> Result<(&str, usize), ()> {
//    //let mut index: u8 = 0;
//    let (length, index) = readVarInt(bytes)?;
//    match std::str::from_utf8(&bytes[index as usize .. (length + 1) as usize]) {
//        Err(_) => return Err(()),
//        Ok(str) => return Ok((str, index as usize + length as usize)) 
//    };
//}
//
//fn readUnsignedShort(bytes: &[u8]) -> u16 {
//    return ((bytes[0] as u16) << 8) | bytes[1] as u16;
//}
