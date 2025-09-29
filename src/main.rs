use clap::Parser;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name="dirhash", about="Scan directory, compute SHA-256, find duplicates")]
struct Args {
    /// Directory to scan
    path: PathBuf,
    /// Only files with these extensions (comma-separated, e.g. "jpg,png,mp4")
    #[arg(long, default_value="")]
    exts: String,
    /// Output duplicates only
    #[arg(long, default_value_t=false)]
    dupes: bool,
}

#[derive(Clone)]
struct Rec {
    size: u64,
    hash: String,
    path: String,
}

fn main() {
    let args = Args::parse();
    let mut files: Vec<PathBuf> = Vec::new();
    let filter_exts: Option<Vec<String>> = if args.exts.trim().is_empty() {
        None
    } else {
        Some(args.exts.split(',').map(|s| s.trim().to_lowercase()).collect())
    };

    for e in WalkDir::new(&args.path).into_iter().filter_map(|e| e.ok()) {
        let p = e.path();
        if p.is_file() {
            if let Some(ref exts) = filter_exts {
                if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
                    if !exts.contains(&ext.to_lowercase()) { continue; }
                } else { continue; }
            }
            files.push(p.to_path_buf());
        }
    }

    let recs: Vec<Rec> = files.par_iter().filter_map(|p| {
        let mut f = File::open(p).ok()?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 1024 * 128];
        let mut size: u64 = 0;
        loop {
            let n = f.read(&mut buf).ok()?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
            size += n as u64;
        }
        let hash = hex::encode(hasher.finalize());
        Some(Rec {
            size,
            hash,
            path: p.to_string_lossy().to_string(),
        })
    }).collect();

    if args.dupes {
        use std::collections::HashMap;
        let mut byhash: HashMap<(u64,String), Vec<&Rec>> = HashMap::new();
        for r in &recs {
            byhash.entry((r.size, r.hash.clone())).or_default().push(r);
        }
        println!("size,sha256,path");
        for ((_size, _h), v) in byhash.into_iter() {
            if v.len() > 1 {
                for r in v {
                    println!("{},{},"{}"", r.size, r.hash, r.path.replace(""",""""));
                }
            }
        }
    } else {
        println!("size,sha256,path");
        for r in recs {
            println!("{},{},"{}"", r.size, r.hash, r.path.replace(""",""""));
        }
    }
}
