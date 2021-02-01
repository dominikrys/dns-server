type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: rename from BytePacketBuffer? Change buf and pos to full words?
pub struct BytePacketBuffer {
    // TODO: do all these need to be pub?
    pub buf: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> Self {
        BytePacketBuffer {
            buf: [0; 512],
            pos: 0,
        }
    }

    fn pos(&self) -> usize {
        self.pos
    }

    // TODO: remove pub
    pub fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps; // TODO: Can this not overflow?

        Ok(()) // TODO: Should we return a proper error?
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos; // TODO: Can this not overflow?

        Ok(()) // TODO: Should we return a proper error?
    }

    // TODO: Rename to "read and advance" or split up into methods
    fn read(&mut self) -> Result<u8> {
        if self.pos >= 512 {
            // TODO: Can we store the size in a constant?
            // TODO: add type for this kind of error?
            return Err("End of buffer".into());
        }

        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    // TODO: does self need to be mutable?
    // TODO: do we need to specify pos or can we just return the current position?
    fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= 512 {
            return Err("End of buffer".into());
        }
        Ok(self.buf[pos])
    }

    // TODO: does self need to be mutable?
    fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        // TODO: maybe refactor this range check
        if start + len >= 512 {
            return Err("End of buffer".into());
        }

        Ok(&self.buf[start..start + len as usize])
    }

    // TODO: does self need to be mutable?
    // TODO: remove pub
    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(((self.read()? as u16) << 8) | (self.read()? as u16))
    }

    // TODO: does self need to be mutable?
    // TODO: remove pub
    pub fn read_u32(&mut self) -> Result<u32> {
        // TODO: use read_16 here? Or generalise for arbitrary values?
        Ok(((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | (self.read()? as u32))
    }

    // TODO: does self need to be mutable?
    // TODO: can just return the string and not output it in an input?
    // TODO: rename from qname to something better. Maybe domain? Or Question name?
    // TODO: remove pub
    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
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

            if (len & 0xC0) == 0xC0 {
                // TODO: get rid of these parentheses
                if !jumped {
                    // TODO: explain that we're adding 2 because the len field is 2 bytes in size
                    self.seek(pos + 2)?; // TODO: can we use len to seek here? Seeking to pos isn't very intuitive
                }

                let b2 = self.get(pos + 1)? as u16; // TODO: rename to "next_byte". This is the next byte of the length
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;

                continue; // TODO: is this redundant
            } else {
                pos += 1;

                if len == 0 {
                    break;
                }

                outstr.push_str(delim);

                let str_buffer = self.get_range(pos, len as usize)?;
                outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase()); // TODO: try without lossy, then fall back to lossy if it fails

                delim = "."; // TODO: do delim more cleanly

                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }
}
