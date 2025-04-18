// audit related definitions

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

const AUDIT_FILE: &'static str = "audit.txt";

// this will hold all audit figures
#[derive(Debug)]
pub struct AuditFile {
    writer: BufWriter<File>,
}

impl AuditFile {
    // create a new instance of the audit file
    pub fn new() -> anyhow::Result<Self> {
        let f = File::create(AUDIT_FILE)?;

        Ok(Self {
            writer: BufWriter::new(f),
        })
    }

    // add metadata
    pub fn add_metadata<P: AsRef<Path>>(&mut self, path: P, length: usize) -> anyhow::Result<()> {
        write!(
            self.writer,
            "image name: {}, file length: {}\n\n",
            path.as_ref().display(),
            length
        )?;

        Ok(())
    }

    // add new data
    pub fn add_artefact<'a>(&mut self, data: &AuditData<'a>) -> anyhow::Result<()> {
        write!(
            self.writer,
            "{}: {}-{} (0x{:X?}-0x{:X?}) {}\n",
            data.artefact,
            data.offset_start,
            data.offset_end,
            data.offset_start,
            data.offset_end,
            data.length
        )?;
        self.writer.flush()?;

        Ok(())
    }
}

// interesting data to know for each artefact
pub struct AuditData<'a> {
    // artefact name
    pub artefact: &'a str,

    // starting offset in the image file
    pub offset_start: u64,

    // end offset in the image file
    pub offset_end: u64,

    // arteffact length
    pub length: u64,
}
