 use core::fmt;
use std::io::{Error, ErrorKind};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{carvers::fourcc_carver::FourCCCarver, deserializer::Deserializer, err};

// start of image
const SOI: [u8; 2] = [0xFF, 0xD8];

// end of image
const EOI: [u8; 2] = [0xFF, 0xD9];

// temporary
const TIM: [u8; 2] = [0xFF, 0x01];

// start of scan => specific processing
const SOS: [u8; 2] = [0xFF, 0xDA];

#[derive(Debug, Default)]
pub struct PNGHeader {
    signature: u64,
}

impl Deserializer for PNGHeader {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        self.signature = buffer.read_u64::<BigEndian>()?;

        Ok(())
    }
}

#[derive(Default)]
pub struct JpegSegment {
    segment_type: [u8; 2], // chunk type
    length: Option<u16>,   // length of the chunk data (big-endian) include itself
}

impl JpegSegment {
    // those segment have no length
    pub fn is_standalone(&self) -> bool {
        if self.segment_type == SOI
            || self.segment_type == EOI
            || self.segment_type == TIM
            || (self.segment_type[0] == 0xFF
                && self.segment_type[1] >= 0xD0
                && self.segment_type[1] <= 0xD7)
        {
            true
        } else {
            false
        }
    }
}

impl fmt::Debug for JpegSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ctype={:X?} length={:?} ",
            self.segment_type, self.length
        )
    }
}

impl Deserializer for JpegSegment {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        self.segment_type[0] = buffer.read_u8()?;
        self.segment_type[1] = buffer.read_u8()?;

        // check for marker validity
        if self.segment_type[0] != 0xFF {
            return err!(ErrorKind::InvalidData);
        }

        // those markers have no length, so return
        if self.is_standalone() {
            return Ok(());
        }

        // all JPEG markers have the second byte > 0xC0 except for 0x01 (TIM) which is already processed
        if self.segment_type[1] < 0xC0 {
            return err!(ErrorKind::InvalidData);
        }

        // SOS entails a specific processing
        // to find the next segment after the SOS, you must keep reading until you find a 0xFF bytes which
        // is not immediately followed by 0x00 (see "byte stuffing")
        if self.segment_type == SOS {
            loop {
                let byte1 = buffer.read_u8()?;

                // search for FF
                if byte1 != 0xFF {
                    continue;
                }

                // found FF: if byte2 is a restart marker (D0 to D7) or 0, continue
                let byte2 = buffer.read_u8()?;
                if (0xD0 <= byte2 && byte2 <= 0xD7) || byte2 == 0 {
                    continue;
                } else {
                    // rewind of 2 bytes
                    buffer.set_position(buffer.position() - 2);
                    return Ok(());
                }
            }
        }

        // if we reached here, this mean it's a segment with a length

        // get length
        self.length = Some(buffer.read_u16::<BigEndian>()?);

        // skip payload
        let pos = buffer.position();
        buffer.set_position(pos + self.length.unwrap() as u64 - 2);

        Ok(())
    }
}

impl FourCCCarver for JpegSegment {
    fn is_end(&self) -> bool {
        self.segment_type == EOI
    }
}
