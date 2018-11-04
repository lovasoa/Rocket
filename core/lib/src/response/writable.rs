use std::io::{copy, Read, Result, Write};

/// Trait implemented by types that can be written to an io::Writer
pub trait Writable {
    /// Write the object to the given writer
    fn write<'r>(
        &mut self,
        // The box is needed in order to make the function callable from a trait object (E0038)
        destination: Box<&mut (Write + 'r)>,
    ) -> Result<()>;

    /// Write the object to the given writer, with a `chunk_size` hint
    /// indicating the preferred number of bytes to write at a time.
    ///
    /// The default implementation ignores the chunk_size.
    fn write_chunked<'r>(
        &mut self,
        destination: Box<&mut (Write + 'r)>,
        chunk_size: usize,
    ) -> Result<()> {
        self.write(destination)
    }

    /// Return a new writable that writes only the first `size` bytes of the current object
    fn take(self, size: usize) -> Take<Self>
        where Self: Sized {
        Take { source: self, size }
    }
}

impl<T: Read> Writable for T {
    fn write<'r>(&mut self, destination: Box<&mut (Write + 'r)>) -> Result<()> {
        copy(self, &mut *destination).map(|_size| ())
    }

    fn write_chunked<'r>(&mut self, destination: Box<&mut (Write + 'r)>, chunk_size: usize) -> Result<()> {
        let mut buffer = vec![0u8; chunk_size];
        loop {
            match self.read(&mut buffer)? {
                0 => break,
                n => { (*destination).write(&buffer[..n]); }
            }
        }
        Ok(())
    }
}

/// The first n bytes of a writable
struct Take<W: Writable> { source: W, size: usize }

impl<W: Writable> Writable for Take<W> {
    fn write<'r>(&mut self, destination: Box<&mut (Write + 'r)>) -> Result<()> {
        let mut buffer = vec![0u8; self.size];
        self.source.write(&mut buffer)?;
        (*destination).write_all(&mut buffer)
    }
}