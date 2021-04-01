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
            return Err(format!(
                "Position of buffer {} exceeds max size of {}",
                pos, BUF_SIZE
            )
            .into());
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

    pub fn read_compressed_name(&mut self) -> Result<String> {
        let mut name = String::new();
        let mut pos = self.pos();

        let mut jumps_performed = 0;
        let max_jumps = 5; // Guard against cycles

        loop {
            if jumps_performed > max_jumps {
                return Err(format!("Limit of {} jumps exceeded", max_jumps).into());
            }

            let label_len = self.get(pos)?;

            let two_msb_mask = 0xC0;
            let pointer = (label_len & two_msb_mask) == two_msb_mask;
            if pointer {
                if jumps_performed == 0 {
                    // Advance past the current label
                    self.seek(pos + 2);
                }

                let offset_b1 = (label_len as u16) ^ (two_msb_mask as u16);
                let offset_b2 = self.get(pos + 1)? as u16;
                let offset = (offset_b1 << 8) | offset_b2;
                pos = offset as usize;

                jumps_performed += 1;
            } else {
                pos += 1;

                if label_len == 0 {
                    break;
                }

                if !name.is_empty() {
                    name.push('.');
                }

                let str_buffer = self.get_range(pos, label_len as usize)?;
                name.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

                pos += label_len as usize;
            }
        }

        if jumps_performed == 0 {
            self.seek(pos);
        }

        Ok(name)
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

    pub fn write_compressed_name(&mut self, name: &str) -> Result<()> {
        for label in name.split('.') {
            let len = label.len();

            let label_len_limit = 63;
            if len > label_len_limit {
                return Err(
                    format!("One label exceeds max length of {} chars", label_len_limit).into(),
                );
            }

            self.write_u8(len as u8)?;

            for &b in label.as_bytes() {
                self.write_u8(b)?;
            }
        }

        self.write_u8(0)?; // Null terminate the name

        Ok(())
    }

    fn set_u8(&mut self, pos: usize, val: u8) -> Result<()> {
        self.check_end_of_buf(pos)?;

        self.buf[pos] = val;

        Ok(())
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) -> Result<()> {
        self.set_u8(pos, (val >> 8) as u8)?;
        self.set_u8(pos + 1, (val & 0xFF) as u8)?;

        Ok(())
    }
}
