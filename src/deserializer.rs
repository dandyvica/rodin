// Deserialize from bigendian or littleendian bytes

pub trait Deserializer {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()>;
}
