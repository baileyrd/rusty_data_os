//! Canonical exchange results. Timing diagnostics live in adjacent results.cmt.
use crate::{
    Id,
    format::{id, unhex},
    hex,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Results {
    pub input_sha256: String,
    pub engine: String,
    pub durability: String,
    pub operations: Vec<String>,
    pub records: Vec<(Id, String)>,
    pub columns: Vec<String>,
}
impl Results {
    pub fn encode(&self) -> String {
        let mut s = header(self, self.operations.len(), self.records.len());
        for (i, op) in self.operations.iter().enumerate() {
            s.push_str(&format!("op\t{}\t{}\n", i + 1, hex(op.as_bytes())));
        }
        for (id, digest) in &self.records {
            s.push_str(&format!("record\t{}\t{digest}\n", hex(id)));
        }
        for (i, digest) in self.columns.iter().enumerate() {
            s.push_str(&format!("column\t{i}\t{digest}\n"));
        }
        s
    }
    pub fn decode(s: &str) -> Result<Self, String> {
        let mut lines = s.lines();
        let h: Vec<_> = lines.next().ok_or("results header")?.split('\t').collect();
        if h.len() != 6 || h[0] != "CMT1-results" || h[1] != "1" {
            return Err("results version/header".into());
        }
        let check_digest = |s: &str| -> Result<String, String> {
            if unhex(s)?.len() != 32 {
                return Err("SHA256 length".into());
            }
            Ok(s.into())
        };
        let input_sha256 = check_digest(h[2])?;
        let engine = String::from_utf8(unhex(h[3])?).map_err(|e| e.to_string())?;
        let count = h[4].parse::<usize>().map_err(|_| "result count")?;
        let record_count = h[5].parse::<usize>().map_err(|_| "record count")?;
        let durability = lines
            .next()
            .and_then(|line| line.strip_prefix("durability\t"))
            .ok_or("durability label")?;
        let durability = String::from_utf8(unhex(durability)?).map_err(|e| e.to_string())?;
        if durability.is_empty() {
            return Err("empty durability label".into());
        }
        let mut result = Self {
            input_sha256,
            engine,
            durability,
            operations: vec![],
            records: vec![],
            columns: vec![],
        };
        for line in lines {
            let p: Vec<_> = line.split('\t').collect();
            if p.len() != 3 {
                return Err("result arity".into());
            }
            match p[0] {
                "op" if result.records.is_empty()
                    && result.columns.is_empty()
                    && p[1] == (result.operations.len() + 1).to_string() =>
                {
                    result
                        .operations
                        .push(String::from_utf8(unhex(p[2])?).map_err(|e| e.to_string())?)
                }
                "record" if result.operations.len() == count && result.columns.is_empty() => {
                    result.records.push((id(p[1])?, check_digest(p[2])?))
                }
                "column"
                    if result.records.len() == record_count
                        && result.operations.len() == count
                        && p[1] == result.columns.len().to_string() =>
                {
                    result.columns.push(check_digest(p[2])?)
                }
                _ => return Err("result ordering".into()),
            }
        }
        if result.operations.len() != count
            || result.records.len() != record_count
            || result.columns.len() != 13
            || result.records.windows(2).any(|w| w[0].0 >= w[1].0)
            || result.encode() != s
        {
            return Err("result counts/order/canonical bytes".into());
        }
        Ok(result)
    }
}

fn header(result: &Results, operations: usize, records: usize) -> String {
    format!(
        "CMT1-results\t1\t{}\t{}\t{operations}\t{records}\ndurability\t{}\n",
        result.input_sha256,
        hex(result.engine.as_bytes()),
        hex(result.durability.as_bytes())
    )
}

/// Streams operations directly to an exclusive observations file. The header
/// reserves the maximum decimal record-count width until actual state is known.
/// Finish compacts that reservation with bounded memory; interrupted output is
/// incomplete and must not be treated as a canonical completed result.
pub struct StreamingResults {
    file: std::fs::File,
    metadata: Results,
    count: usize,
    written: usize,
    reserved: u64,
}
impl StreamingResults {
    pub fn create(
        path: &std::path::Path,
        input_sha256: String,
        engine: String,
        durability: String,
        count: usize,
    ) -> Result<Self, String> {
        use std::io::Write;
        if durability.is_empty() {
            return Err("empty durability label".into());
        }
        let metadata = Results {
            input_sha256,
            engine,
            durability,
            operations: vec![],
            records: vec![],
            columns: vec![],
        };
        let prefix = header(&metadata, count, usize::MAX);
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.write_all(prefix.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(Self {
            file,
            metadata,
            count,
            written: 0,
            reserved: prefix.len() as u64,
        })
    }
    pub fn operation(&mut self, signature: &str) -> Result<(), String> {
        use std::io::Write;
        if self.written == self.count {
            return Err("too many observed operations".into());
        }
        writeln!(
            self.file,
            "op\t{}\t{}",
            self.written + 1,
            hex(signature.as_bytes())
        )
        .map_err(|e| e.to_string())?;
        self.written += 1;
        Ok(())
    }
    pub fn finish(mut self, records: &[(Id, String)], columns: &[String]) -> Result<(), String> {
        use std::io::{Read, Seek, SeekFrom, Write};
        if self.written != self.count {
            return Err("missing observed operations".into());
        }
        let mut finish = || -> std::io::Result<()> {
            for (id, digest) in records {
                writeln!(self.file, "record\t{}\t{digest}", hex(id))?;
            }
            for (i, digest) in columns.iter().enumerate() {
                writeln!(self.file, "column\t{i}\t{digest}")?;
            }
            let end = self.file.stream_position()?;
            let prefix = header(&self.metadata, self.count, records.len());
            let gap = self.reserved - prefix.len() as u64;
            let mut read_at = self.reserved;
            let mut buffer = [0_u8; 65536];
            // Copy forward: each destination precedes its source, and the full
            // chunk is read before overlapping writes. No operation vector lives here.
            while read_at < end && gap > 0 {
                let n = (end - read_at).min(buffer.len() as u64) as usize;
                self.file.seek(SeekFrom::Start(read_at))?;
                self.file.read_exact(&mut buffer[..n])?;
                self.file.seek(SeekFrom::Start(read_at - gap))?;
                self.file.write_all(&buffer[..n])?;
                read_at += n as u64;
            }
            self.file.set_len(end - gap)?;
            self.file.seek(SeekFrom::Start(0))?;
            self.file.write_all(prefix.as_bytes())
        };
        finish().map_err(|e| e.to_string())
    }
}
