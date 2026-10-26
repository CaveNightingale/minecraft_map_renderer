use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail, ensure};
use libdeflater::{DecompressionError, Decompressor};
use lz4_java_wrc::Lz4BlockInput;

const SECTOR_SIZE: u64 = 4096;
const DEFAULT_CACHE_BYTES: usize = 1024 * 1024 * 1024;
const INITIAL_CHUNK_BYTES: usize = 128 * 1024;
const MAX_CHUNK_BYTES: usize = 64 * 1024 * 1024;

/// Reuses one libdeflate instance and one output buffer across chunks.
pub struct Decoder {
    decompressor: Decompressor,
    buffer: Vec<u8>,
    external: Vec<u8>,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            decompressor: Decompressor::new(),
            buffer: vec![0; INITIAL_CHUNK_BYTES],
            external: Vec::new(),
        }
    }

    fn decode(&mut self, compression: u8, payload: &[u8]) -> Result<&[u8]> {
        match compression {
            1 | 2 => {
                let size = loop {
                    let result = if compression == 2 {
                        self.decompressor.zlib_decompress(payload, &mut self.buffer)
                    } else {
                        self.decompressor.gzip_decompress(payload, &mut self.buffer)
                    };
                    match result {
                        Ok(size) => break size,
                        Err(DecompressionError::InsufficientSpace) => {
                            let size = self.buffer.len() * 2;
                            ensure!(size <= MAX_CHUNK_BYTES, "chunk payload is too large");
                            self.buffer.resize(size, 0);
                        }
                        Err(error) => bail!("inflating chunk payload: {error:?}"),
                    }
                };
                Ok(&self.buffer[..size])
            }
            4 => {
                self.external.clear();
                Lz4BlockInput::new(payload).read_to_end(&mut self.external)?;
                Ok(&self.external)
            }
            other => bail!("unsupported region compression type {other}"),
        }
    }

    fn store_external(&mut self, data: Vec<u8>) -> &[u8] {
        self.external = data;
        &self.external
    }
}

#[derive(Debug)]
pub struct Location {
    pub region: PathBuf,
    pub x: i32,
    pub z: i32,
    offset: u64,
    sectors: u8,
}

pub fn scan(input: &Path) -> Result<Vec<Location>> {
    let mut chunks = BTreeMap::new();
    for entry in fs::read_dir(input)
        .with_context(|| format!("reading region directory {}", input.display()))?
    {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "mca") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("non-UTF-8 region filename")?;
        let parts: Vec<_> = name.split('.').collect();
        ensure!(
            parts.len() == 4 && parts[0] == "r",
            "invalid region filename {name}"
        );
        let rx: i32 = parts[1]
            .parse()
            .with_context(|| format!("invalid region x in {name}"))?;
        let rz: i32 = parts[2]
            .parse()
            .with_context(|| format!("invalid region z in {name}"))?;
        let mut file =
            File::open(&path).with_context(|| format!("opening region {}", path.display()))?;
        let length = file
            .metadata()
            .with_context(|| format!("stat of region {}", path.display()))?
            .len();
        if length == 0 {
            continue;
        }
        ensure!(
            length >= 8192,
            "truncated region header in {}",
            path.display()
        );
        let mut header = [0_u8; 4096];
        file.read_exact(&mut header)
            .with_context(|| format!("reading header of region {}", path.display()))?;
        let mut occupied = BTreeMap::new();
        for index in 0..1024 {
            let bytes = &header[index * 4..index * 4 + 4];
            let entry = u32::from_be_bytes(bytes.try_into().expect("four bytes"));
            if entry == 0 {
                continue;
            }
            let sector = u64::from(entry >> 8);
            let sectors = entry as u8;
            ensure!(
                sector >= 2
                    && sectors > 0
                    && sector + u64::from(sectors) <= length.div_ceil(SECTOR_SIZE),
                "invalid chunk location {index} in {}",
                path.display()
            );
            for number in sector..sector + u64::from(sectors) {
                ensure!(
                    occupied.insert(number, index).is_none(),
                    "overlapping chunks in {}",
                    path.display()
                );
            }
            let x = rx
                .checked_mul(32)
                .and_then(|base| base.checked_add((index % 32) as i32))
                .context("region x coordinate overflow")?;
            let z = rz
                .checked_mul(32)
                .and_then(|base| base.checked_add((index / 32) as i32))
                .context("region z coordinate overflow")?;
            ensure!(
                chunks
                    .insert(
                        (x, z),
                        Location {
                            region: path.clone(),
                            x,
                            z,
                            offset: sector * SECTOR_SIZE,
                            sectors,
                        }
                    )
                    .is_none(),
                "duplicate chunk at {x},{z}"
            );
        }
    }
    Ok(chunks.into_values().collect())
}

pub struct RegionFile {
    path: PathBuf,
    data: Vec<u8>,
}

impl RegionFile {
    pub fn read(path: &Path) -> Result<Self> {
        let data = fs::read(path).with_context(|| format!("reading region {}", path.display()))?;
        ensure!(
            data.len() >= 8192,
            "truncated region header in {}",
            path.display()
        );
        Ok(Self {
            path: path.to_owned(),
            data,
        })
    }

    /// Decodes one chunk, reusing `decoder`'s buffers so a whole render pass
    /// allocates one output buffer instead of one per chunk.
    pub fn chunk_into<'a>(
        &'a self,
        location: &Location,
        decoder: &'a mut Decoder,
    ) -> Result<&'a [u8]> {
        self.chunk_inner(location, decoder).with_context(|| {
            format!(
                "reading chunk {},{} in {}",
                location.x,
                location.z,
                self.path.display()
            )
        })
    }

    fn chunk_inner<'a>(
        &'a self,
        location: &Location,
        decoder: &'a mut Decoder,
    ) -> Result<&'a [u8]> {
        let offset = usize::try_from(location.offset).context("region offset is too large")?;
        let header_end = offset.checked_add(5).context("region offset overflow")?;
        ensure!(header_end <= self.data.len(), "truncated chunk header");
        let header = &self.data[offset..header_end];
        let length = u32::from_be_bytes(header[..4].try_into().expect("four bytes"));
        ensure!(length >= 1, "chunk length excludes the compression byte");
        ensure!(
            u64::from(length) + 4 <= u64::from(location.sectors) * SECTOR_SIZE,
            "chunk length exceeds its allocated sectors"
        );
        let payload_end = offset
            .checked_add(4 + length as usize)
            .context("chunk length overflow")?;
        ensure!(payload_end <= self.data.len(), "truncated chunk payload");
        let compression = header[4] & 0x7f;
        if header[4] & 0x80 != 0 {
            ensure!(
                length == 1 && location.sectors == 1,
                "invalid external chunk stub"
            );
            let path = self
                .path
                .with_file_name(format!("c.{}.{}.mcc", location.x, location.z));
            let external = fs::read(&path)
                .with_context(|| format!("reading external chunk {}", path.display()))?;
            return if compression == 3 {
                Ok(decoder.store_external(external))
            } else {
                decoder.decode(compression, &external)
            };
        }
        let payload = &self.data[offset + 5..payload_end];
        if compression == 3 {
            return Ok(payload);
        }
        decoder.decode(compression, payload)
    }
}

impl Location {
    pub fn read(&self, region: &RegionFile) -> Result<Vec<u8>> {
        let mut decoder = Decoder::new();
        Ok(region.chunk_into(self, &mut decoder)?.to_vec())
    }
}

pub(crate) struct RegionCache {
    budget: usize,
    used: usize,
    files: HashMap<PathBuf, Arc<RegionFile>>,
    order: VecDeque<PathBuf>,
}

impl Default for RegionCache {
    fn default() -> Self {
        Self {
            budget: DEFAULT_CACHE_BYTES,
            used: 0,
            files: HashMap::new(),
            order: VecDeque::new(),
        }
    }
}

impl RegionCache {
    pub fn get(&mut self, path: &Path) -> Result<Arc<RegionFile>> {
        if let Some(file) = self.files.get(path).cloned() {
            self.order.retain(|cached| cached != path);
            self.order.push_back(path.to_owned());
            return Ok(file);
        }
        let file = Arc::new(RegionFile::read(path)?);
        let size = file.data.len();
        if size <= self.budget {
            while self.used.saturating_add(size) > self.budget {
                let Some(oldest) = self.order.pop_front() else {
                    break;
                };
                if let Some(old) = self.files.remove(&oldest) {
                    self.used -= old.data.len();
                }
            }
            self.used += size;
            self.files.insert(path.to_owned(), file.clone());
            self.order.push_back(path.to_owned());
        }
        Ok(file)
    }
}
