// Deserialize from bigendian or littleendian bytes

use std::io::Cursor;

// useful macro for deserializing
#[macro_export]
macro_rules! err {
    ($e:path) => {
        Err(Error::from($e))
    };
}

pub trait Deserializer {
    fn deserialize(&mut self, buffer: &mut Cursor<&[u8]>) -> std::io::Result<usize>;
}
