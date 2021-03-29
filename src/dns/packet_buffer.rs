type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const BUF_SIZE: usize = 512;
pub struct PacketBuffer {
    buf: [u8; BUF_SIZE],
    pos: usize,
}

impl PacketBuffer {
    pub fn new() -> Self {
        PacketBuffer {
            buf: [0; BUF_SIZE],
            pos: 0,
        }
    }

    pub fn from_u8_array(buf: [u8; BUF_SIZE]) -> Self {
        PacketBuffer { buf, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps: usize) {
        self.pos += steps;
    }

    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn check_end_of_buf(&self, pos: usize) -> Result<()> {
        if pos >= self.buf.len() {
            return Err("End of buffer".into());
        }

        Ok(())
    }

    fn get(&self, pos: usize) -> Result<u8> {
        self.check_end_of_buf(pos)?;

        Ok(self.buf[pos])
    }

    pub fn get_range(&self, start: usize, len: usize) -> Result<&[u8]> {
        self.check_end_of_buf(start + len)?;

        Ok(&self.buf[start..start + len as usize])
    }

    fn read_u8(&mut self) -> Result<u8> {
        let res = self.get(self.pos)?;
        self.pos += 1;

        Ok(res)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(((self.read_u8()? as u16) << 8) | (self.read_u8()? as u16))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(((self.read_u16()? as u32) << 16) | (self.read_u16()? as u32))
    }

    pub fn read_qname(&mut self) -> Result<String> {
        let mut qname = String::new();

        let mut pos = self.pos(); // TODO: maybe make more obvious that this is local

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut delim = "";

        loop {
            if jumps_performed > max_jumps {
                return Err(format!("Limit of {} jumps exceeded", max_jumps).into());
            }

            let len = self.get(pos)?; // TODO: rename to label_len

            // Check if the two most significant bits are set: https://docstore.mik.ua/orelly/networking_2ndEd/dns/ch15_02.htm
            let two_msb_mask = 0xC0;

            if (len & two_msb_mask) == two_msb_mask {
                // TODO: get rid of these parentheses
                if !jumped {
                    // TODO: explain that we're adding 2 because the len field is 2 bytes in size
                    self.seek(pos + 2); // TODO: can we use len to seek here? Seeking to pos isn't very intuitive
                }

                let b2 = self.get(pos + 1)? as u16; // TODO: rename to "next_byte". This is the next byte of the length
                let offset = (((len as u16) ^ two_msb_mask as u16) << 8) | b2;
                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;
            } else {
                pos += 1;

                if len == 0 {
                    break;
                }

                qname.push_str(delim);

                let str_buffer = self.get_range(pos, len as usize)?;
                qname.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase()); // TODO: try without lossy, then fall back to lossy if it fails

                delim = "."; // TODO: do delim more cleanly

                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos);
        }

        Ok(qname)
    }

    pub fn write_u8(&mut self, val: u8) -> Result<()> {
        self.check_end_of_buf(self.pos)?;

        self.buf[self.pos] = val;
        self.pos += 1;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write_u8((val >> 8) as u8)?;
        self.write_u8((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write_u16((val >> 16) as u16)?;
        self.write_u16((val & 0xFFFF) as u16)?;

        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 63 {
                return Err("Single label exceeds 63 characters of length".into());
                // TODO: should we restore the whole buffer in this case?
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                // TODO: can we avoid the * here and instead use a reference?
                self.write_u8(*b)?;
            }
        }

        // Null terminate the name
        self.write_u8(0)?;

        Ok(())
    }
}
