use anyhow::{Context, Result, bail, ensure};

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

const MAX_DEPTH: usize = 512;

#[derive(Clone, Copy, Debug)]
pub struct LongArray<'a> {
    bytes: &'a [u8],
}

impl<'a> LongArray<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        ensure!(
            bytes.len().is_multiple_of(8),
            "NBT long array length is not a multiple of 8"
        );
        Ok(Self { bytes })
    }

    pub fn len(&self) -> usize {
        self.bytes.len() / 8
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn get(&self, index: usize) -> i64 {
        let start = index * 8;
        i64::from_be_bytes(
            self.bytes[start..start + 8]
                .try_into()
                .expect("eight bytes"),
        )
    }
}

pub struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .context("NBT offset overflow")?;
        ensure!(end <= self.data.len(), "unexpected end of NBT data");
        let slice = &self.data[self.position..end];
        self.position = end;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn i8(&mut self) -> Result<i8> {
        Ok(self.take(1)?[0] as i8)
    }

    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }

    pub fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    pub fn string_bytes(&mut self) -> Result<&'a [u8]> {
        let length = usize::from(self.u16()?);
        self.take(length)
    }

    pub fn long_array(&mut self) -> Result<LongArray<'a>> {
        let length = self.i32()?;
        let length = usize::try_from(length).context("negative NBT long array length")?;
        let length = length.checked_mul(8).context("NBT long array overflow")?;
        LongArray::new(self.take(length)?)
    }

    pub fn compound<F>(&mut self, mut field: F) -> Result<()>
    where
        F: FnMut(&mut Self, &'a [u8], u8) -> Result<()>,
    {
        loop {
            let tag = self.u8()?;
            if tag == TAG_END {
                return Ok(());
            }
            let name = self.string_bytes()?;
            field(self, name, tag)?;
        }
    }

    pub fn list<F>(&mut self, mut element: F) -> Result<()>
    where
        F: FnMut(&mut Self, u8) -> Result<()>,
    {
        let tag = self.u8()?;
        let length = self.i32()?;
        let length = usize::try_from(length).context("negative NBT list length")?;
        ensure!(
            tag != TAG_END || length == 0,
            "nonempty NBT list of end tags"
        );
        for _ in 0..length {
            element(self, tag)?;
        }
        Ok(())
    }

    pub fn skip(&mut self, tag: u8) -> Result<()> {
        self.skip_inner(tag, 0)
    }

    fn skip_inner(&mut self, tag: u8, depth: usize) -> Result<()> {
        ensure!(depth <= MAX_DEPTH, "NBT nesting is too deep");
        match tag {
            TAG_END => bail!("unexpected NBT end tag"),
            TAG_BYTE => {
                self.take(1)?;
            }
            TAG_SHORT => {
                self.take(2)?;
            }
            TAG_INT | TAG_FLOAT => {
                self.take(4)?;
            }
            TAG_LONG | TAG_DOUBLE => {
                self.take(8)?;
            }
            TAG_BYTE_ARRAY | TAG_INT_ARRAY | TAG_LONG_ARRAY => {
                let length = self.i32()?;
                let width = match tag {
                    TAG_BYTE_ARRAY => 1,
                    TAG_INT_ARRAY => 4,
                    _ => 8,
                };
                let length = usize::try_from(length).context("negative NBT array length")?;
                let bytes = length
                    .checked_mul(width)
                    .context("NBT array length overflow")?;
                self.take(bytes)?;
            }
            TAG_STRING => {
                self.string_bytes()?;
            }
            TAG_LIST => self.list(|reader, element| reader.skip_inner(element, depth + 1))?,
            TAG_COMPOUND => {
                self.compound(|reader, _name, field| reader.skip_inner(field, depth + 1))?
            }
            other => bail!("unknown NBT tag type {other}"),
        }
        Ok(())
    }
}
