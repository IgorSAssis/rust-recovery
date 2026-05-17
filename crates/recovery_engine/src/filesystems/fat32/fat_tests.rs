use std::io::Cursor;

use super::{collect_cluster_chain, read_fat_entry, FAT_EOC_MIN};
use crate::filesystems::fat32::{Fat32BootSector, test_helpers::build_image};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Boot sector matching the geometry produced by `build_image`.
fn boot() -> Fat32BootSector {
    Fat32BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 4,
        num_fats: 1,
        total_sectors_32: 16,
        fat_size_32: 2,
        root_cluster: 2,
    }
}

// ── read_fat_entry ────────────────────────────────────────────────────────────

#[test]
fn read_fat_entry_returns_eoc_for_root_cluster() {
    // Cluster 2 = root dir → FAT entry should be EOC (0x0FFFFFFF)
    let img = build_image(&[(2, 0x0FFF_FFFF)], &[]);
    let mut cur = Cursor::new(img);
    let entry = read_fat_entry(&mut cur, &boot(), 2).unwrap();
    assert_eq!(entry, 0x0FFF_FFFF);
}

#[test]
fn read_fat_entry_masks_upper_4_bits() {
    // Write a value with upper 4 bits set; they must be masked out.
    let img = build_image(&[(3, 0xF000_0005)], &[]);
    let mut cur = Cursor::new(img);
    let entry = read_fat_entry(&mut cur, &boot(), 3).unwrap();
    assert_eq!(entry, 0x0000_0005);
}

#[test]
fn read_fat_entry_returns_zero_for_free_cluster() {
    let img = build_image(&[], &[]); // cluster 3 is left as 0
    let mut cur = Cursor::new(img);
    let entry = read_fat_entry(&mut cur, &boot(), 3).unwrap();
    assert_eq!(entry, 0);
}

#[test]
fn read_fat_entry_for_cluster_0_returns_media_marker() {
    let img = build_image(&[], &[]);
    let mut cur = Cursor::new(img);
    // FAT[0] is always set to the media descriptor (0x0FFFFFF8 after masking)
    let entry = read_fat_entry(&mut cur, &boot(), 0).unwrap();
    assert_eq!(entry, 0x0FFF_FFF8);
}

// ── collect_cluster_chain ─────────────────────────────────────────────────────

#[test]
fn collect_cluster_chain_single_cluster_eoc() {
    // Chain: 2 → EOC
    let img = build_image(&[(2, 0x0FFF_FFFF)], &[]);
    let mut cur = Cursor::new(img);
    let chain = collect_cluster_chain(&mut cur, &boot(), 2).unwrap();
    assert_eq!(chain, vec![2]);
}

#[test]
fn collect_cluster_chain_two_clusters() {
    // Chain: 2 → 3 → EOC
    let img = build_image(&[(2, 3), (3, 0x0FFF_FFFF)], &[]);
    let mut cur = Cursor::new(img);
    let chain = collect_cluster_chain(&mut cur, &boot(), 2).unwrap();
    assert_eq!(chain, vec![2, 3]);
}

#[test]
fn collect_cluster_chain_three_clusters_in_order() {
    // Chain: 4 → 5 → 6 → EOC
    let img = build_image(&[(4, 5), (5, 6), (6, 0x0FFF_FFFF)], &[]);
    let mut cur = Cursor::new(img);
    let chain = collect_cluster_chain(&mut cur, &boot(), 4).unwrap();
    assert_eq!(chain, vec![4, 5, 6]);
}

#[test]
fn collect_cluster_chain_stops_at_free_cluster() {
    // Chain: 2 → 3 → 0 (free) — should stop after cluster 3
    let img = build_image(&[(2, 3), (3, 0)], &[]);
    let mut cur = Cursor::new(img);
    let chain = collect_cluster_chain(&mut cur, &boot(), 2).unwrap();
    assert_eq!(chain, vec![2, 3]);
}

#[test]
fn collect_cluster_chain_eoc_boundary_value_included() {
    // FAT_EOC_MIN itself must be treated as end-of-chain
    let img = build_image(&[(2, FAT_EOC_MIN)], &[]);
    let mut cur = Cursor::new(img);
    let chain = collect_cluster_chain(&mut cur, &boot(), 2).unwrap();
    assert_eq!(chain, vec![2]);
}
