// Pass 1: a frame-granularity sweep of the shared script (src/lib.rs). For
// every WRAM/HRAM byte, record which frames it changed on and what values it
// took. Cheap: `Gameboy::frame()` is the same high-level stepping the
// existing TerminalGB harness uses.
use gen1_observe::run_script;
use std::collections::BTreeMap;
use std::io::Write;
use std::time::Instant;
use terminalgb::gameboy::Gameboy;

const VRAM_LO: u16 = 0x8000;
const VRAM_HI: u16 = 0x9FFF;
const WRAM_LO: u16 = 0xC000;
const WRAM_HI: u16 = 0xDFFF;
const HRAM_LO: u16 = 0xFF80;
const HRAM_HI: u16 = 0xFFFE;

#[derive(Default)]
struct ByteHistory {
    change_frames: Vec<u32>,
    values_seen: Vec<u8>,
}

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let out_path = std::env::var("OBSERVE_OUT").unwrap_or_else(|_| "observe-pass1.json".into());

    let data = std::fs::read(&rom_path).expect("read rom");
    let mut gb = match save_path {
        Some(ref p) => {
            let staged_rom = "target/observe-staged.gb";
            let staged_sav = "target/observe-staged.sav";
            std::fs::create_dir_all("target").ok();
            std::fs::copy(&rom_path, staged_rom).unwrap();
            std::fs::copy(p, staged_sav).unwrap();
            Gameboy::new(
                std::fs::read(staged_rom).unwrap(),
                Some(std::path::PathBuf::from(staged_rom)),
            )
        }
        None => Gameboy::new(data, None),
    };

    let mut history: BTreeMap<u16, ByteHistory> = BTreeMap::new();
    let mut prev: BTreeMap<u16, u8> = BTreeMap::new();
    for (lo, hi) in [(VRAM_LO, VRAM_HI), (WRAM_LO, WRAM_HI), (HRAM_LO, HRAM_HI)] {
        for a in lo..=hi {
            prev.insert(a, gb.peek(a));
        }
    }
    let mut co_occurrence: Vec<(u32, Vec<u16>)> = Vec::new();

    let start = Instant::now();
    let mut last_frame = 0u32;
    {
        let mut on_frame = |gb: &mut Gameboy, frame_no: u32| {
            gb.frame();
            last_frame = frame_no;
            let mut changed_this_frame = Vec::new();
            for (lo, hi) in [(VRAM_LO, VRAM_HI), (WRAM_LO, WRAM_HI), (HRAM_LO, HRAM_HI)] {
                let now = gb.peek_range(lo, (hi - lo) as usize + 1);
                for (i, b) in now.iter().enumerate() {
                    let addr = lo + i as u16;
                    let before = prev.get(&addr).copied().unwrap_or(0);
                    if *b != before {
                        let h = history.entry(addr).or_default();
                        if h.change_frames.len() < 96 {
                            h.change_frames.push(frame_no);
                        }
                        if !h.values_seen.contains(b) && h.values_seen.len() < 32 {
                            h.values_seen.push(*b);
                        }
                        changed_this_frame.push(addr);
                        prev.insert(addr, *b);
                    }
                }
            }
            if !changed_this_frame.is_empty() && co_occurrence.len() < 30000 {
                co_occurrence.push((frame_no, changed_this_frame));
            }
        };
        run_script(&mut gb, &mut on_frame);
    }

    let elapsed = start.elapsed();
    eprintln!(
        "ran {} frames in {:.2}s ({} bytes with at least one observed change, {} co-occurrence frame-events)",
        last_frame + 1,
        elapsed.as_secs_f64(),
        history.len(),
        co_occurrence.len()
    );

    let mut out = std::fs::File::create(&out_path).unwrap();
    write!(out, "{{\"total_frames\": {}, \"addresses\": {{", last_frame + 1).unwrap();
    let mut first = true;
    for (addr, h) in &history {
        if !first {
            write!(out, ",").unwrap();
        }
        first = false;
        write!(
            out,
            "\"{:04X}\": {{\"change_frames\": {:?}, \"values_seen\": {:?}}}",
            addr, h.change_frames, h.values_seen
        )
        .unwrap();
    }
    write!(out, "}}, \"co_occurrence\": [").unwrap();
    for (i, (f, addrs)) in co_occurrence.iter().enumerate() {
        if i > 0 {
            write!(out, ",").unwrap();
        }
        let addr_strs: Vec<String> = addrs.iter().map(|a| format!("\"{a:04X}\"")).collect();
        write!(out, "[{f}, [{}]]", addr_strs.join(",")).unwrap();
    }
    writeln!(out, "]}}").unwrap();
    eprintln!("wrote {out_path}");
}
