use core::fmt;
use std::{
    io::{BufRead, Error, ErrorKind, Read},
    ops::Deref,
};

use byteorder::{BigEndian, ReadBytesExt};
use log::trace;

use crate::{carvers::fourcc_carver::FourCCCarver, deserializer::Deserializer, err};

// common JPEG segments

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

// a segment type is given by just 2 bytes
#[derive(Debug, Default)]
pub struct SegmentType([u8; 2]);

impl SegmentType {
    // those segment have no length
    pub fn is_standalone(&self) -> bool {
        if self.0 == SOI
            || self.0 == EOI
            || self.0 == TIM
            || (self.0[0] == 0xFF && self.0[1] >= 0xD0 && self.0[1] <= 0xD7)
        {
            true
        } else {
            false
        }
    }

    // not all arrays of 2 bytes are valid Jpeg segments
    fn is_valid(&self) -> bool {
        // we only consider 2-bytes
        if self.0.len() != 2 {
            return false;
        }

        // first byte must be 0xFF
        if self.0[0] != 0xFF {
            return false;
        }

        // now second byte
        if self.0[1] < 0xC0 {
            return false;
        }

        true
    }
}

impl Deref for SegmentType {
    type Target = [u8; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<[u8; 2]> for SegmentType {
    fn eq(&self, other: &[u8; 2]) -> bool {
        self.0 == *other
    }
}

#[derive(Default)]
pub struct JpegSegment {
    segment_type: SegmentType, // chunk type
    length: Option<u16>,       // length of the chunk data (big-endian) include itself
}

impl JpegSegment {
    // those segment have no length
    pub fn is_standalone(&self) -> bool {
        self.segment_type.is_standalone()
    }
}

impl fmt::Debug for JpegSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "segment_type={:X?} length={:?} ",
            self.segment_type, self.length
        )
    }
}

impl Deserializer for JpegSegment {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        // read first byte
        self.segment_type.0[0] = buffer.read_u8()?;

        // check for marker validity
        if self.segment_type[0] != 0xFF {
            return err!(ErrorKind::InvalidData);
        }

        // now we can read the second byte
        self.segment_type.0[1] = buffer.read_u8()?;

        // those markers have no length, so return
        if self.is_standalone() {
            trace!(
                "standalone segment found: {:?}, offset: {}",
                self,
                buffer.position()
            );
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
                let mut buf = vec![];

                // read until we find 0xFF
                let _ = buffer.read_until(0xFF, &mut buf)?;

                // let byte1 = buffer.read_u8()?;

                // // search for FF
                // if byte1 != 0xFF {
                //     continue;
                // }

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
