//! Replays the Rust CLI against goldens frozen from the TypeScript CLI.
//! No node is needed; the goldens are frozen.

use std::fs;
use std::path::{Path, PathBuf};

/// Minimal SHA-256 (FIPS 180-4) so the golden checks need no extra dependency.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

/// Verifies every `hash  path` line of `dir/SHA256SUMS` and returns the count.
fn verify_sums(dir: &std::path::Path) -> usize {
    let sums = std::fs::read_to_string(dir.join("SHA256SUMS")).expect("SHA256SUMS");
    let mut n = 0;
    for line in sums.lines() {
        let (hash, rel) = line.split_once("  ").expect("malformed SHA256SUMS line");
        let bytes = std::fs::read(dir.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert_eq!(
            sha256_hex(&bytes),
            hash,
            "{rel} changed; goldens are frozen, update SHA256SUMS only for an intended change"
        );
        n += 1;
    }
    n
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_tree(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn files(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            files(&p, base, out);
        } else {
            out.push(p.strip_prefix(base).unwrap().to_path_buf());
        }
    }
}

#[test]
fn rust_cli_matches_frozen_typescript_goldens() {
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    assert!(verify_sums(&golden) > 0);
    for mode in ["check", "fix"] {
        let tmp = tempfile::TempDir::new().unwrap();
        copy_tree(&golden.join("vault"), tmp.path());
        let out = std::process::Command::new(assert_cmd::cargo::cargo_bin("lynter"))
            .current_dir(tmp.path())
            .args([mode, ".", "--json", "--today", "2026-01-01"])
            .output()
            .unwrap();
        let want_code: i32 = fs::read_to_string(golden.join(format!("{mode}.code")))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert_eq!(out.status.code().unwrap(), want_code, "{mode} exit code");
        let want_out = fs::read(golden.join(format!("{mode}.stdout"))).unwrap();
        assert!(!want_out.is_empty());
        assert_eq!(
            String::from_utf8(out.stdout).unwrap(),
            String::from_utf8(want_out).unwrap(),
            "{mode} --json output"
        );
        let mut rels = Vec::new();
        files(&golden.join(mode), &golden.join(mode), &mut rels);
        assert!(rels.len() >= 4);
        for rel in rels {
            assert_eq!(
                fs::read(tmp.path().join(&rel)).unwrap(),
                fs::read(golden.join(mode).join(&rel)).unwrap(),
                "{mode} content of {}",
                rel.display()
            );
        }
    }
}
