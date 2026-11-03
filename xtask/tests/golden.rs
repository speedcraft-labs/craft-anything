use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

fn temporary(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let dir = std::env::temp_dir().join(format!("craft-anything-xtask-{label}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    let clean: Vec<u8> = hex.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    clean.as_chunks::<2>().0.iter().map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("ascii hex"), 16).expect("hex byte")).collect()
}

fn fake_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0, 1, 2, 3, 4]);
    bytes
}

#[test]
fn ico_matches_original_bytes_and_silent_stdout() {
    let temp = temporary("ico");
    let out = temp.join("test.ico");
    let pngs = ["assets/app-icon/hicolor/16x16/apps/ai.storyteller.craft-anything.png", "assets/app-icon/hicolor/32x32/apps/ai.storyteller.craft-anything.png"];
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(root())
        .args(["ico", out.to_str().expect("utf8 path"), pngs[0], pngs[1]])
        .output()
        .expect("run xtask ico");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, include_bytes!("golden/ico.stdout"));
    let expected = hex_bytes(include_str!("golden/ico.hex"));
    assert_eq!(std::fs::read(&out).expect("read out.ico"), expected);

    let cfg = craft_xtask::ico::Config { root: root(), ..Default::default() };
    let report = craft_xtask::ico::compute(
        &cfg,
        &craft_xtask::ico::Args { output: Some(PathBuf::from("test.ico")), inputs: pngs.into_iter().map(PathBuf::from).collect() },
    )
    .expect("compute ico");
    assert_eq!(report.bytes, expected);
    assert_eq!(report.sizes, vec![16, 32]);
    std::fs::remove_dir_all(temp).expect("cleanup temp");
}

#[test]
fn ico_configured_paths_and_committed_icon_reproduce_identically() {
    let cfg = craft_xtask::ico::Config {
        root: root(),
        source_pngs: [16, 24, 32, 48, 64, 128, 256]
            .into_iter()
            .map(|s| PathBuf::from(format!("assets/app-icon/hicolor/{s}x{s}/apps/ai.storyteller.craft-anything.png")))
            .collect(),
        output_path: Some(PathBuf::from("assets/app-icon/craft-anything.ico")),
        sizes: Vec::new(),
        icns: false,
    };
    let report = craft_xtask::ico::compute(&cfg, &craft_xtask::ico::Args::default()).expect("compute configured ico");
    assert_eq!(report.sizes, vec![16, 24, 32, 48, 64, 128, 256]);
    assert_eq!(&report.bytes[..6], &[0, 0, 1, 0, 7, 0]);

    let committed = std::fs::read(root().join("assets/app-icon/craft-anything.ico")).expect("read committed ico");
    assert_eq!(&committed[..4], &[0, 0, 1, 0]);
    let count = u16::from_le_bytes([committed[4], committed[5]]) as usize;
    assert_eq!(count, 9);
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let dir = &committed[6 + i * 16..6 + (i + 1) * 16];
        let len = u32::from_le_bytes([dir[8], dir[9], dir[10], dir[11]]) as usize;
        let off = u32::from_le_bytes([dir[12], dir[13], dir[14], dir[15]]) as usize;
        entries.push(committed[off..off + len].to_vec());
    }
    assert_eq!(craft_xtask::ico::pack_ico(&entries).expect("repack committed ico"), committed);
}

#[test]
fn ico_rejects_malformed_images_and_missing_arguments() {
    assert!(craft_xtask::ico::pack_ico(&[]).is_err());
    assert!(craft_xtask::ico::pack_ico(&[b"nope".to_vec()]).is_err());
    for (w, h) in [(0, 0), (16, 32), (512, 512)] {
        assert!(craft_xtask::ico::pack_ico(&[fake_png(w, h)]).is_err(), "{w}x{h}");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_xtask")).current_dir(root()).arg("ico").output().expect("run xtask ico without args");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: cargo xtask ico <out.ico> <png>..."));
}

#[test]
fn help_includes_shared_task_syntax() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask")).current_dir(root()).arg("--help").output().expect("run xtask --help");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in ["layers", "locks", "i18n-coverage", "secrets", "version", "ico", craft_xtask::ico::cli_help()] {
        assert!(stderr.contains(expected), "missing {expected} in {stderr}");
    }
}
