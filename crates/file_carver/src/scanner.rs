use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};

use crate::carved_file::CarvedFile;
use crate::signature::{FileKind, Signature};

struct InProgress {
    kind: FileKind,
    start_abs: u64,
}

/// Scans `source` for all known file signatures using a sliding-window strategy.
///
/// Reading is done in chunks of `chunk_size` bytes. Consecutive chunks overlap
/// by `max_pattern_length - 1` bytes so that signatures spanning a chunk
/// boundary are never missed.
///
/// Returns carved files sorted by `offset_start`.
pub fn scan<R: Read + Seek>(
    source: &mut R,
    signatures: &[&Signature],
    chunk_size: usize,
) -> std::io::Result<Vec<CarvedFile>> {
    if signatures.is_empty() || chunk_size == 0 {
        return Ok(Vec::new());
    }

    // The overlap must be large enough to bridge any pattern across a boundary.
    let max_pattern_len = signatures
        .iter()
        .flat_map(|s| [s.header_pattern.len(), s.footer_pattern.len()])
        .max()
        .unwrap_or(0);
    let overlap_size = max_pattern_len.saturating_sub(1);

    source.seek(SeekFrom::Start(0))?;

    let mut carved: Vec<CarvedFile> = Vec::new();
    let mut in_progress: Vec<InProgress> = Vec::new();
    // Tracks absolute offsets of headers already registered to prevent the
    // overlap region from producing duplicate entries on the next iteration.
    let mut known_headers: HashSet<u64> = HashSet::new();

    let mut overlap: Vec<u8> = Vec::new();
    let mut chunk = vec![0u8; chunk_size];
    let mut disk_pos: u64 = 0; // byte position in source BEFORE the current read

    loop {
        let bytes_read = source.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }

        let new_bytes = &chunk[..bytes_read];

        // Build the scanning window: tail of the previous chunk + new bytes.
        // window[0] maps to absolute disk offset `window_base`.
        let mut window = overlap.clone();
        window.extend_from_slice(new_bytes);
        let window_base = disk_pos.saturating_sub(overlap.len() as u64);

        // ── Step 1 ──────────────────────────────────────────────────────────
        // Close in-progress files whose footer now appears in this window.
        // A per-kind cursor ensures multiple same-type files consume footers
        // in FIFO order without reusing the same footer bytes twice.
        let mut footer_cursors: HashMap<FileKind, usize> = HashMap::new();
        let mut still_open: Vec<InProgress> = Vec::new();

        for ip in in_progress {
            let fp = footer_for(signatures, ip.kind);
            let from = *footer_cursors.get(&ip.kind).unwrap_or(&0);

            match find_pattern(&window, fp, from) {
                Some(i) => {
                    carved.push(CarvedFile {
                        kind: ip.kind,
                        offset_start: ip.start_abs,
                        offset_end: window_base + i as u64 + fp.len() as u64,
                    });
                    footer_cursors.insert(ip.kind, i + fp.len());
                }
                None => still_open.push(ip),
            }
        }
        in_progress = still_open;

        // ── Step 2 ──────────────────────────────────────────────────────────
        // Scan the entire window for new headers.
        // Deduplication via `known_headers` prevents re-processing bytes that
        // appeared in the previous window's overlap region.
        for sig in signatures {
            let hp = sig.header_pattern;
            if hp.is_empty() {
                continue;
            }

            for local_idx in 0..=window.len().saturating_sub(hp.len()) {
                if &window[local_idx..local_idx + hp.len()] != hp {
                    continue;
                }

                let header_abs = window_base + local_idx as u64;
                if known_headers.contains(&header_abs) {
                    continue;
                }
                known_headers.insert(header_abs);

                let fp = sig.footer_pattern;
                let footer_from = local_idx + hp.len();

                match find_pattern(&window, fp, footer_from) {
                    Some(j) => carved.push(CarvedFile {
                        kind: sig.kind,
                        offset_start: header_abs,
                        offset_end: window_base + j as u64 + fp.len() as u64,
                    }),
                    None => in_progress.push(InProgress {
                        kind: sig.kind,
                        start_abs: header_abs,
                    }),
                }
            }
        }

        disk_pos += bytes_read as u64;

        // Keep the last `overlap_size` bytes of the new data for the next window.
        let tail_start = new_bytes.len().saturating_sub(overlap_size);
        overlap = new_bytes[tail_start..].to_vec();
    }

    carved.sort_by_key(|f| f.offset_start);
    Ok(carved)
}

/// Searches `window` for `pattern` starting at byte index `from`.
/// Returns the index of the first match, or `None`.
fn find_pattern(window: &[u8], pattern: &[u8], from: usize) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let last_start = window.len().saturating_sub(pattern.len());
    if from > last_start {
        return None;
    }
    for i in from..=last_start {
        if &window[i..i + pattern.len()] == pattern {
            return Some(i);
        }
    }
    None
}

/// Returns the footer pattern for `kind` from the provided signature list.
fn footer_for<'a>(signatures: &[&'a Signature], kind: FileKind) -> &'a [u8] {
    signatures
        .iter()
        .find(|s| s.kind == kind)
        .map(|s| s.footer_pattern)
        .unwrap_or(&[])
}
