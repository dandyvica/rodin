use core::fmt;
use std::io::{Error, ErrorKind};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{carvers::fourcc_carver::FourCCCarver, deserializer::Deserializer};

const SOI: [u8; 2] = [0xFF, 0xD8];
const EOI: [u8; 2] = [0xFF, 0xD9];

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
    seg_type: [u8; 2],   // chunk type
    length: Option<u16>, // length of the chunk data (big-endian) include itself
}

impl fmt::Debug for JpegSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ctype={:X?} length={:?} ", self.seg_type, self.length)
    }
}

impl Deserializer for JpegSegment {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        self.seg_type[0] = buffer.read_u8()?;
        self.seg_type[1] = buffer.read_u8()?;

        // if Start of Image or End Of Image, no length so return
        if self.seg_type == SOI || self.seg_type == EOI {
            return Ok(());
        }

        // SOS entails a specific processing
        // to find the next segment after the SOS, you must keep reading until you find a 0xFF bytes which
        // is not immediately followed by 0x00 (see "byte stuffing")
        if self.seg_type == SOS {
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

        // get length
        self.length = Some(buffer.read_u16::<BigEndian>()?);

        // skip payload
        let pos = buffer.position();
        buffer.set_position(pos + self.length.unwrap() as u64 - 2);

        // compare against usual chunk types
        // if self.chunk_type != *b"IHDR"
        //     && self.chunk_type != *b"PLTE"
        //     && self.chunk_type != *b"IDAT"
        //     && self.chunk_type != *b"IEND"
        //     && self.chunk_type != *b"tEXt"
        //     && self.chunk_type != *b"iTXt"
        //     && self.chunk_type != *b"tIME"
        //     && self.chunk_type != *b"gAMA"
        //     && self.chunk_type != *b"sRGB"
        //     && self.chunk_type != *b"iCCP"
        //     && self.chunk_type != *b"pHYs"
        // {
        //     return Err(Error::from(ErrorKind::InvalidData));
        // }
        // println!("pos={}", buffer.position());

        Ok(())
    }
}

impl FourCCCarver for JpegSegment {
    fn is_end(&self) -> bool {
        self.seg_type == EOI
    }
}
