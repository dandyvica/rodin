use byteorder::{LittleEndian, ReadBytesExt};

use crate::{carvers::size_carver::SizeCarver, deserializer::Deserializer};

// see: https://www.ece.ualberta.ca/~elliott/ee552/studentAppNotes/2003_w/misc/bmp_file_format/bmp_file_format.htm
#[derive(Debug, Default)]
pub struct BMP {
    magic: u16,             // should be 'BM'
    size: u32,              // bitmap size in little endian
    zeroes: u32,            // should be == 0
    pixel_data_offset: u32, // the address of the image data
    // end of Bitmap fi,le header
    // start of BITMAPI,NFOHEADER or BITMAPV4HEADER or BITMAPV5HEADER
    dib_size: u32, // the size of the DIB header structure. Could be 40 (BITMAPINFOHEADER), 56 (BITMAPV3INFOHEADER), 128 (BITMAPV5HEADER), 108 (BITMAPV4HEADER)
    width: u32,    // image width
    height: u32,   // image height
    bi_planes: u16, // should be 1 if BITMAPINFOHEADER
    bi_bit_count: u16, // bits per pixel
    bi_compression: u32, // the compression method being used
    bi_size_image: u32, // the image size
    bi_xpels_per_meter: u32, // the horizontal resolution of the image. (pixel per metre, signed integer)
    bi_ypels_per_meter: u32, // the horizontal resolution of the image. (pixel per metre, signed integer)
    bi_clr_used: u32,        // the number of colors in the color palette, or 0 to default to 2n
    bi_clr_important: u32, // the number of important colors used, or 0 when every color is important; generally ignored
}

impl SizeCarver for BMP {
    fn size(&self) -> usize {
        self.size as usize
    }

    fn is_genuine(&self) -> bool {
        const BITMAPINFOHEADER: u32 = 40;
        const BITMAPV3INFOHEADER: u32 = 56;
        const BITMAPV4HEADER: u32 = 108;
        const BITMAPV5HEADER: u32 = 128;

        self.zeroes == 0
            && (self.dib_size == BITMAPINFOHEADER
                || self.dib_size == BITMAPV5HEADER
                || self.dib_size == BITMAPV4HEADER
                || self.dib_size == BITMAPV3INFOHEADER)
    }

    fn ext(&self) -> String {
        String::from("bmp")
    }
}

impl Deserializer for BMP {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<usize> {
        self.magic = buffer.read_u16::<LittleEndian>()?;
        self.size = buffer.read_u32::<LittleEndian>()?;
        self.zeroes = buffer.read_u32::<LittleEndian>()?;
        self.pixel_data_offset = buffer.read_u32::<LittleEndian>()?;
        self.dib_size = buffer.read_u32::<LittleEndian>()?;
        self.width = buffer.read_u32::<LittleEndian>()?;
        self.height = buffer.read_u32::<LittleEndian>()?;
        self.bi_planes = buffer.read_u16::<LittleEndian>()?;
        self.bi_bit_count = buffer.read_u16::<LittleEndian>()?;
        self.bi_compression = buffer.read_u32::<LittleEndian>()?;
        self.bi_size_image = buffer.read_u32::<LittleEndian>()?;
        self.bi_xpels_per_meter = buffer.read_u32::<LittleEndian>()?;
        self.bi_ypels_per_meter = buffer.read_u32::<LittleEndian>()?;
        self.bi_clr_used = buffer.read_u32::<LittleEndian>()?;
        self.bi_clr_important = buffer.read_u32::<LittleEndian>()?;

        Ok(48 + 6)
    }
}
