// use failure::Error;

use std::io::Cursor;

#[derive(Debug)]
pub(crate) enum FrameError {
    Incomplete,
}

pub(crate) fn check(src: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
    match get_line(src) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub(crate) fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;

    let end = src.get_ref().len() as usize;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);

            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(FrameError::Incomplete)
}
