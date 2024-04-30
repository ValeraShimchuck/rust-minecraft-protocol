use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt::format;
use std::io::{Read, Write};
use std::io::{Error, Result, ErrorKind};
use std::ops::Deref;
use std::os::windows::io::AsSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use bytebuffer::{ByteBuffer, ByteReader};
use tokio::time::{sleep, Instant};
use uuid::Uuid;
use once_cell::sync::Lazy;
use fastnbt::{Value, nbt};
use serde::{Serialize, Deserialize};

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};


const SEG_BITS: u32 = 0x7F;
const CON_BIT: u32 = 0x80;
const SERVER_ICON: &str = "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";
static REGISTRY: Lazy<Value> = Lazy::new(|| {
    nbt!({
        "minecraft:worldgen/biome": {
            "type": "minecraft:worldgen/biome",
            "value": [
                {
                    "name": "minecraft:plains",
                    "id": 0,
                    "element": {
                        "has_precipitation": 0 as u8,
                        "temperature": 1.0f32,
                        "downfall": 0.5f32,
                        "effects": {
                            "fog_color": 0xE7F1F3,
                            "water_color": 0x356AFA,
                            "water_fog_color": 0x2146AE,
                            "sky_color": 0x9AE1FA
                        }
                    }
                }
            ]
        },
        "minecraft:chat_type": {
            "type": "minecraft:chat_type",
            "value": [
                {
                    "name": "minecraft:default",
                    "id": 0,
                    "element": {
                        "chat": {
                            "translation_key": "chat.type.text",
                            "parameters": [
                                "sender",
                                "content",
                                "target"
                            ]
                        },
                        "narration": {
                            "translation_key": "chat.type.text",
                            "parameters": [
                                "sender",
                                "content",
                                "target"
                            ]
                        }
                    }
                }
            ]
        },
        "minecraft:dimension_type": {
            "type": "minecraft:dimension_type",
            "value": [
                {
                    "name": "minecraft:overworld",
                    "id": 0,
                    "element": {
                        "piglin_safe": 0 as u8,
                        "natural": 1 as u8,
                        "ambient_light": 0.0f32,
                        "monster_spawn_block_light_limit": 0 as u8,
                        "infiniburn": "#minecraft:infiniburn_overworld",
                        "respawn_anchor_works": 0 as u8,
                        "has_skylight": 1 as u8,
                        "bed_works": 1 as u8,
                        "effects": "minecraft:overworld",
                        "has_raids": 1 as u8,
                        "logical_height": 384,
                        "coordinate_scale": 1.0f32,
                        "monster_spawn_light_level": {
                            "type": "minecraft:uniform",
                            "value": {
                                "min_inclusive": 0,
                                "max_inclusive": 7
                            }
                        },
                        "min_y": -64,
                        "ultrawarm": 0 as u8,
                        "has_ceiling": 0 as u8,
                        "height": 384
                        }
                }
            ]
        },
        "minecraft:damage_type": {
            "type": "minecraft:damage_type",
            "value": [
                {
                    "name": "minecraft:arrow",
                    "id": 0,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:in_fire",
                    "id": 1,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:lightning_bolt",
                    "id": 2,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:on_fire",
                    "id": 3,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:lava",
                    "id": 4,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:hot_floor",
                    "id": 5,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:in_wall",
                    "id": 6,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:cramming",
                    "id": 7,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:drown",
                    "id": 8,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:starve",
                    "id": 9,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:cactus",
                    "id": 10,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:fall",
                    "id": 11,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:fly_into_wall",
                    "id": 12,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:out_of_world",
                    "id": 13,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:generic",
                    "id": 14,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:magic",
                    "id": 15,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:wither",
                    "id": 16,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:dragon_breath",
                    "id": 17,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:dry_out",
                    "id": 18,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:sweet_berry_bush",
                    "id": 19,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:freeze",
                    "id": 20,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:stalagmite",
                    "id": 21,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:outside_border",
                    "id": 22,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:generic_kill",
                    "id": 23,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                },
                {
                    "name": "minecraft:player_attack",
                    "id": 24,
                    "element": {
                        "scaling": "when_caused_by_living_non_player",
                        "exhaustion": 0.1f32,
                        "message_id": "arrow"
                    }
                }
            ]
        }
    })
});
//struct ByteBuffer {
//    index: Cell<usize>,
//    buffer: Vec<u8>
//}

static HEIGHT_MAP: Lazy<Value> = Lazy::new(|| {
    nbt!({
        "MOTION_BLOCKING": u64_filled_vec(36),
        "WORLD_SURFACE": u64_filled_vec(36)
    })
});

struct PlayerSample {
    uuid: Uuid,
    name: String
}

#[derive(Serialize)]
struct Registry {



}

impl PlayerSample {
    fn to_json(&self) -> String {
        format!(r#"{{
            "name": "{}",
            "id": "{}"
        }}"#, self.name, self.uuid)
    }

    fn parse_list_to_json(vec: &Vec<PlayerSample>) -> String {
        vec.iter().map(|obj| obj.to_json()).fold("".to_string(), |l, r| l + ",\n" + &r)
    }

}

struct MinecraftStatus {
    version_name: String,
    protocol: u16,
    max_players: u32,
    online: u32,
    sample: Vec<PlayerSample>,
    description: String,
    favicon: String,
    enforces_secure_chat: bool,
    previews_chat: bool
}

static STATUS: Lazy<MinecraftStatus>  = Lazy::new(|| {
    MinecraftStatus {
        version_name: "1.20.4".to_string(),
        protocol: 765,
        max_players: 100,
        online: 0,
        sample: vec![],
        description: "{ \"text\": \"description here\" }".to_string(),
        favicon: SERVER_ICON.to_string(),
        enforces_secure_chat: false,
        previews_chat: false
    }
});

impl MinecraftStatus {

    fn to_json(&self) -> String {
        format!(r#" {{
            "version": {{
                "name": "{}",
                "protocol": {}
            }},
            "players": {{
                "max": {},
                "online": {},
                "sample": [
                    {}
                ]
            }},
            "description": {},
            "favicon": "data:image/png;base64,{}",
            "enforcesSecureChat": {},
            "previewsChat": {}
        }}
        "#, self.version_name, self.protocol, self.max_players, self.online,
         PlayerSample::parse_list_to_json(&self.sample), self.description, self.favicon, self.enforces_secure_chat, self.previews_chat)
    }

}

trait MinecraftReadTypes {
    fn read_var_int(&mut self) -> std::io::Result<u32>;

    
    fn read_var_string(&mut self) -> std::io::Result<String>;

    fn readabe_bytes(&self) -> usize;

    fn read_uuid(&mut self) -> Result<Uuid>;
}

trait MinecraftWriteTypes {
    
    fn write_var_int(&mut self, int: u32);

    fn write_var_string(&mut self, str: &str);

    fn write_uuid(&mut self, uuid: &Uuid);

    fn write_compound(&mut self, nbt: &Value);

}

impl MinecraftWriteTypes for ByteBuffer {
    fn write_var_int(&mut self, mut int: u32) {
        loop {
            if (int & !SEG_BITS) == 0 {
                self.write_u8(int as u8);
                return;
            }
            self.write_u8(((int & SEG_BITS) | CON_BIT) as u8);
            int = int >> 7;
        }
    }

    fn write_var_string(&mut self, str: &str) {
        self.write_var_int(str.len() as u32);
        self.write_bytes(str.as_bytes());
    }


    
    fn write_uuid(&mut self, uuid: &Uuid) {
        let (most_sig_bits, least_sig_bits) = uuid.as_u64_pair();
        self.write_u64(most_sig_bits);
        self.write_u64(least_sig_bits);
    }
    
    fn write_compound(&mut self, nbt: &Value) {
        let mut data = fastnbt::to_bytes(nbt).unwrap();
        data.swap(2, 0);
        self.write_bytes(&data[2..]);
    }
}

impl MinecraftReadTypes for ByteBuffer {


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
    
    fn readabe_bytes(&self) -> usize {
        return self.get_wpos() - self.get_rpos();
    }
    
    fn read_uuid(&mut self) -> Result<Uuid> {
        let most_sig_bits = self.read_u64()?;
        let least_sig_bits = self.read_u64()?;
        Ok(Uuid::from_u64_pair(most_sig_bits, least_sig_bits))
    }
    


}


fn now() -> u128 {
    return  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("starting server");
    let listener = TcpListener::bind("127.0.0.1:25565").await?;
    println!("started server");
    loop {
        let (mut socket, _) = listener.accept().await?;
        let mut state: u8 = 0;
        tokio::spawn(async move {
            let mut accamulated_buffer = ByteBuffer::from_vec(Vec::with_capacity(8192));
            accamulated_buffer.set_wpos(0);
            let mut buffer = vec![0;2048];
            let mut split_packets: VecDeque<ByteBuffer> = VecDeque::new();
            let mut last_keepalive: u128 = now();
            loop {
                let current_time = now();
                if current_time - last_keepalive > 10000 && state == 4 {
                    let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x24);
                    content_write_buffer.write_u64(now() as u64);
                    if !write_packet(&mut socket, &mut content_write_buffer).await {
                        return;
                    }
                    last_keepalive = now();
                }
                
                let n = match socket.try_read(&mut buffer) {
                    Ok(n) => n,
                    Err(ref e) => if e.kind() == io::ErrorKind::WouldBlock {
                        usize::MAX
                    } else {
                        panic!("{e}");
                    }
                };
                if n == usize::MAX {
                    sleep(Duration::from_millis(50)).await;
                    continue;
                }
                if n == 0 {
                    return;
                }
                //let mut read_buffer: ByteReader = ByteReader::from_bytes(&buffer);
                // framing
                accamulated_buffer.write_bytes(&buffer[..n]);
                while accamulated_buffer.readabe_bytes() > 0 {
                    let bytes_to_read = accamulated_buffer.readabe_bytes();
                    let reader_marker = accamulated_buffer.get_rpos();
                    let packet_length = match accamulated_buffer.read_var_int() {
                        Ok(n) => n,
                        Err(e) => {
                            if e.kind() == ErrorKind::UnexpectedEof {
                                accamulated_buffer.set_rpos(reader_marker); // varint isnt full
                                break;
                            }
                            println!("{e}");
                            return;
                        }
                    };
                    if packet_length as usize > accamulated_buffer.readabe_bytes() {
                        accamulated_buffer.set_rpos(reader_marker);
                        break;
                    }
                    split_packets.push_back(ByteBuffer::from_vec(accamulated_buffer.read_bytes(packet_length as usize).unwrap()));

                    // discard read bytes
                    let bytes: Vec<u8> = accamulated_buffer.read_bytes(accamulated_buffer.readabe_bytes()).unwrap();
                    accamulated_buffer.set_rpos(0);
                    accamulated_buffer.set_wpos(0);
                    accamulated_buffer.write_all(&bytes).unwrap();

                }
                for packet_buffer in split_packets.iter_mut() {
                    let packet_id = packet_buffer.read_var_int().unwrap();
                    if packet_id == 0 && state == 0 {
                        let protocol_version = packet_buffer.read_var_int().unwrap();
                        let server_address = packet_buffer.read_var_string().unwrap();
                        let server_port = packet_buffer.read_u16().unwrap();
                        let game_state = packet_buffer.read_var_int().unwrap();
                        state = game_state as u8;
                    }
                    
                    else if packet_id == 0 && state == 1 {
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0);
                        content_write_buffer.write_var_string(&STATUS.to_json());
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }
                    }

                    else if packet_id == 1 && state == 1 {
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(1);
                        content_write_buffer.write_u64(packet_buffer.read_u64().unwrap());
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }
                    }

                    else if packet_id == 0 && state == 2 {
                        let name = packet_buffer.read_var_string().unwrap();
                        let uuid = packet_buffer.read_uuid().unwrap();

                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(2);
                        content_write_buffer.write_uuid(&uuid);
                        content_write_buffer.write_var_string(&name);
                        content_write_buffer.write_var_int(0);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }
                    }
                    else if packet_id == 3 && state == 2 {
                        state = 3;
                        let mut data = fastnbt::to_bytes(REGISTRY.deref()).unwrap();
                        data.swap(2, 0);
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(5);
                        content_write_buffer.write_bytes(&data[2..]);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(2);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }
                        //println!("registry {}",  data.iter().fold("".to_string(), |mut left, right| {left.push_str(&format!("|{:#04X}", right)); left}));
                    } else if packet_id == 2 && state == 3 {
                        state = 4;
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x29);
                        content_write_buffer.write_u32(0);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_var_int(1);
                        content_write_buffer.write_var_string("minecraft:overworld");
                        content_write_buffer.write_var_int(100);
                        content_write_buffer.write_var_int(10);
                        content_write_buffer.write_var_int(10);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_var_string("minecraft:overworld");
                        content_write_buffer.write_var_string("minecraft:overworld");
                        content_write_buffer.write_u64(0);
                        content_write_buffer.write_u8(3);
                        content_write_buffer.write_i8(-1);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_var_int(0);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }

                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x52);
                        content_write_buffer.write_var_int(0);
                        content_write_buffer.write_var_int(0);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }
                        

                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x20);
                        content_write_buffer.write_u8(13);
                        content_write_buffer.write_f32(0.0f32);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }

                        // send chunk

                        for x in -1..=1 {
                            for y in -1..=1 {
                                let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x25);
                                content_write_buffer.write_i32(x);
                                content_write_buffer.write_i32(y);
                                content_write_buffer.write_compound(&HEIGHT_MAP);
                                content_write_buffer.write_var_int(192);
                                for _i in 0..384/16 {
                                    content_write_buffer.write_u16(0);
                                    for _j in 0..2 {
                                        content_write_buffer.write_var_int(0);
                                        content_write_buffer.write_var_int(0);
                                        content_write_buffer.write_var_int(0);
                                    }
                                }
                                content_write_buffer.write_var_int(0);
                                // light
                                content_write_buffer.write_var_int(0);
                                content_write_buffer.write_var_int(0);
                                content_write_buffer.write_var_int(0);
                                content_write_buffer.write_var_int(0);
                                content_write_buffer.write_var_int(0);
                                content_write_buffer.write_var_int(0);
                            
                                if !write_packet(&mut socket, &mut content_write_buffer).await {
                                    return;
                                }
                            }
                        }
                        

                        // teleport player
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x3E);
                        content_write_buffer.write_f64(0.5f64);
                        content_write_buffer.write_f64(1.0f64);
                        content_write_buffer.write_f64(0.5f64);
                        content_write_buffer.write_f32(0.0f32);
                        content_write_buffer.write_f32(0.0f32);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_var_int(0);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }

                        // set block
                        //let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x09);
                        //content_write_buffer.write_u64(0);
                        //content_write_buffer.write_var_int(1);
                        //if !write_packet(&mut socket, &mut content_write_buffer).await {
                        //    return;
                        //}
//
                        //
                        //let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x09);
                        //content_write_buffer.write_u64(2 | (1 << 38));
                        //content_write_buffer.write_var_int(12514);
                        //if !write_packet(&mut socket, &mut content_write_buffer).await {
                        //    return;
                        //}
                        if !write_block(&mut socket, 0, 1, 0, 1).await {
                            return;
                        };
                        for x in -2..=2 {
                            for z in -2..=2 {
                                if !write_block(&mut socket, x, 3, z, 7406).await {
                                    return;
                                };
                                if !write_block(&mut socket, x, 4, z, 1).await {
                                    return;
                                };
                            }
                        }
                        //if !write_block(&mut socket, 0, 2, 1, 12514).await {
                        //    return;
                        //};
                        //if !write_block(&mut socket, 0, 3, 0, 12514).await {
                        //    return;
                        //};
                        //if !write_block(&mut socket, 1, 2, 0, 12514).await {
                        //    return;
                        //};
                        //if !write_block(&mut socket, 0, 2, -1, 12514).await {
                        //    return;
                        //};
                        //if !write_block(&mut socket, -1, 2, 0, 12514).await {
                        //    return;
                        //};
                        //if !write_block(&mut socket, 0, 1, 0, 12514).await {
                        //    return;
                        //};

                        // send armorstand with player's entity id
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x01);
                        content_write_buffer.write_var_int(1);
                        let entity_uuid = Uuid::new_v4();
                        content_write_buffer.write_uuid(&entity_uuid);
                        content_write_buffer.write_var_int(2);
                        content_write_buffer.write_f64(0.5f64);
                        content_write_buffer.write_f64(1.0f64);
                        content_write_buffer.write_f64(0.5f64);
                        content_write_buffer.write_i8(-64);
                        content_write_buffer.write_i8(0);
                        content_write_buffer.write_i8(0);
                        content_write_buffer.write_var_int(0);
                        content_write_buffer.write_u16(0);
                        content_write_buffer.write_u16(0);
                        content_write_buffer.write_u16(0);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }

                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x56);
                        content_write_buffer.write_var_int(1);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_u8(0);
                        content_write_buffer.write_u8(0x20);
                        content_write_buffer.write_u8(0xff);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }

                        
                        let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x50);
                        content_write_buffer.write_var_int(1);
                        if !write_packet(&mut socket, &mut content_write_buffer).await {
                            return;
                        }

                    }
                    else if packet_id == 23 && state == 4 {
                        let x = packet_buffer.read_f64().unwrap();
                        let y = packet_buffer.read_f64().unwrap();
                        let z = packet_buffer.read_f64().unwrap();
                        let on_ground = packet_buffer.read_u8().unwrap() == 1;
                    }
                }
                split_packets.clear();
            }
        });
    }
}

async fn write_block<T>(socket: &mut T, x: i32, y: i16, z: i32, block_id: u32) -> bool
where T: AsyncWriteExt + Unpin
{
    let mut content_write_buffer: ByteBuffer = prepare_packet_buffer(0x09);
    content_write_buffer.write_i64((y & 0xfff) as i64 | (((z & 0x3ffffff) as i64) << 12) | (((x &  & 0x3ffffff) as i64) << 38));
    content_write_buffer.write_var_int(block_id);
    if !write_packet(socket, &mut content_write_buffer).await {
        return false;
    }
    return true;
}

fn allocate_buffer() -> ByteBuffer {
    let mut content_write_buffer: ByteBuffer = ByteBuffer::from_vec(vec![0;0]);
    content_write_buffer.set_wpos(0);
    content_write_buffer
}

fn prepare_packet_buffer(packet_id: u32) -> ByteBuffer {
    let mut buffer = allocate_buffer();
    buffer.write_var_int(packet_id);
    buffer
}

async fn write_packet<T>(socket: &mut T, buffer: &mut ByteBuffer) -> bool
where T: AsyncWriteExt + Unpin {
    let mut framed_write_buffer: ByteBuffer = ByteBuffer::from_vec(Vec::new());
    framed_write_buffer.write_var_int(buffer.readabe_bytes() as u32);
    framed_write_buffer.write_all(buffer.as_bytes()).unwrap();
    if let Err(e) = socket.write_all(framed_write_buffer.as_bytes()).await {
        eprintln!("Error {e}");
        return false;
    };
    //framed_write_buffer.write_all(buffer.as_bytes()).unwrap();
    if let Err(e) = socket.flush().await {
        eprintln!("Error {e}");
        return false;
    };
    true
}

fn u64_filled_vec(size: usize) -> Vec<u64> {
    let mut vec = Vec::with_capacity(size);
    vec.resize(size, 0);
    vec
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
