use std::io::Write;
use std::vec::Vec;

trait MultibootTag {
    fn get_type(&self) -> u16;
    fn data(&self) -> Vec<u8>;
}

struct MultibootInfoReq {
    flags: Vec<u32>,
}

impl MultibootTag for MultibootInfoReq {
    fn get_type(&self) -> u16 {
        1
    }
    fn data(&self) -> Vec<u8> {
        self.flags.iter().flat_map(|f| f.to_le_bytes()).collect()
    }
}

struct MultibootConsoleFlags {
    flags: u32,
}

impl MultibootTag for MultibootConsoleFlags {
    fn get_type(&self) -> u16 {
        4
    }
    fn data(&self) -> Vec<u8> {
        self.flags.to_le_bytes().to_vec()
    }
}

struct MultibootFramebufferReq {
    width: u32,
    height: u32,
    depth: u32,
}

impl MultibootTag for MultibootFramebufferReq {
    fn get_type(&self) -> u16 {
        5
    }
    fn data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.width.to_le_bytes());
        data.extend(self.height.to_le_bytes());
        data.extend(self.depth.to_le_bytes());
        data
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Write the multiboot header to a file
    let mut data = Vec::new();

    let tags: Vec<Box<dyn MultibootTag>> = vec![
        Box::new(MultibootInfoReq { flags: vec![8] }),
        /* TODO: impl framebuffer
        Box::new(MultibootConsoleFlags { flags: 0b01 }),
        Box::new(MultibootFramebufferReq {
            width: 80,
            height: 25,
            depth: 0,
        }),
        */
    ];

    for tag in tags {
        data.extend(tag.get_type().to_le_bytes()); // type
        data.extend([0u8; 2].to_vec()); // flags
        let tdata = tag.data();

        let length = tdata.len() + 8;
        data.extend((length as u32).to_le_bytes());
        data.extend(tdata);

        // align to 8 bytes
        let padding = (8 - (length % 8)) % 8;
        data.extend(vec![0u8; padding]);
    }

    // end tag
    data.extend([0u8; 4].to_vec());
    data.extend(8u32.to_le_bytes());

    let mut header = Vec::new();
    header.extend(0xE85250D6u32.to_le_bytes()); // magic
    header.extend(0u32.to_le_bytes()); // i386
    header.extend((16u32 + data.len() as u32).to_le_bytes()); // header length
    let checksum = 0xE85250D6 + 16 + data.len() as u32;
    header.extend((!checksum + 1).to_le_bytes());

    header.extend(data);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let output = std::path::Path::new(&out_dir).join("multiboot.bin");

    let mut file = std::fs::File::create(output).unwrap();
    file.write_all(&header).unwrap();
}
