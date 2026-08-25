// Pass 1: a frame-granularity sweep of a real, richer playthrough than
// TerminalGB's own gen1atlas.rs Tier C script (which never reaches a battle
// or a menu past the opening). For every WRAM/HRAM byte, record which frames
// it changed on and what values it took. Cheap: `Gameboy::frame()` is the
// same high-level stepping the existing harness uses.
use std::collections::BTreeMap;
use std::time::Instant;
use terminalgb::gameboy::Gameboy;
use terminalgb::KeypadKey;

const WRAM_LO: u16 = 0xC000;
const WRAM_HI: u16 = 0xDFFF;
const HRAM_LO: u16 = 0xFF80;
const HRAM_HI: u16 = 0xFFFE;

fn regions() -> Vec<(u16, u16)> {
    vec![(WRAM_LO, WRAM_HI), (HRAM_LO, HRAM_HI)]
}

#[derive(Default)]
struct ByteHistory {
    // frames this byte changed on (capped)
    change_frames: Vec<u32>,
    values_seen: Vec<u8>,
}

fn press(gb: &mut Gameboy, key: KeypadKey, frames_held: u32, frames_gap: u32, sweep: &mut impl FnMut(&mut Gameboy, u32)) {
    gb.keydown(key);
    for _ in 0..frames_held {
        sweep(gb, 1);
    }
    gb.keyup(key);
    for _ in 0..frames_gap {
        sweep(gb, 1);
    }
}

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let out_path = std::env::var("OBSERVE_OUT").unwrap_or_else(|_| "observe-pass1.json".into());

    let data = std::fs::read(&rom_path).expect("read rom");
    let mut gb = match save_path {
        Some(ref p) => {
            // stage a copy so nothing writes beside the real save
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
    for (lo, hi) in regions() {
        for a in lo..=hi {
            prev.insert(a, gb.peek(a));
        }
    }

    let mut frame_no: u32 = 0;
    let mut co_occurrence: Vec<(u32, Vec<u16>)> = Vec::new();

    let mut sweep = |gb: &mut Gameboy, n: u32| {
        for _ in 0..n {
            gb.frame();
            frame_no += 1;
            let mut changed_this_frame = Vec::new();
            for (lo, hi) in regions() {
                let now = gb.peek_range(lo, (hi - lo) as usize + 1);
                for (i, b) in now.iter().enumerate() {
                    let addr = lo + i as u16;
                    let before = prev.get(&addr).copied().unwrap_or(0);
                    if *b != before {
                        let h = history.entry(addr).or_default();
                        if h.change_frames.len() < 64 {
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
            if !changed_this_frame.is_empty() && co_occurrence.len() < 20000 {
                co_occurrence.push((frame_no, changed_this_frame));
            }
        }
    };

    let start = Instant::now();

    // Phase 1: title -> naming/continue screens, same shape as gen1atlas.rs's
    // own script (A and START alternating opens a save, mashes past intro).
    for frame in 0..1400u32 {
        let want = if (frame / 24) % 2 == 0 {
            KeypadKey::A
        } else {
            KeypadKey::Start
        };
        gb.keydown(want);
        sweep(&mut gb, 1);
        gb.keyup(want);
    }

    // Phase 2: walk and open menus (same cadence as gen1atlas.rs Tier C),
    // long enough to reach the overworld reliably and exercise the sprite
    // engine, text engine and map-loading bytes.
    for frame in 0..1800u32 {
        let want = match (frame / 20) % 8 {
            0 | 1 => KeypadKey::Down,
            2 => KeypadKey::A,
            3 => KeypadKey::Up,
            4 => KeypadKey::Left,
            5 => KeypadKey::Start,
            6 => KeypadKey::Right,
            _ => KeypadKey::B,
        };
        gb.keydown(want);
        sweep(&mut gb, 1);
        gb.keyup(want);
    }

    // Phase 3: force a wild battle the way the game's own debug menu does —
    // writing wCurOpponent, an already-verified, fully-described entry in
    // this atlas ($D059) — rather than needing to walk into grass. Species 1
    // internal index, level 5.
    gb.debug_write(0xD059, 1); // wCurOpponent: species internal index 1
    gb.debug_write(0xD127, 5); // wCurEnemyLevel
    for _ in 0..30u32 {
        sweep(&mut gb, 1);
    }
    // Fight it out: mash A. On the main FIGHT/PKMN/ITEM/RUN menu this selects
    // FIGHT; on the move list it selects the first move; on text boxes it
    // advances them. Not clever, consistent — same philosophy as the existing
    // harness script.
    for _ in 0..40u32 {
        let mut s2 = |gb: &mut Gameboy, n: u32| sweep(gb, n);
        press(&mut gb, KeypadKey::A, 8, 16, &mut s2);
    }

    // Phase 4: back in the overworld (win, lose, or run out the clock),
    // open the START menu and walk it — POKEMON, ITEM, and back — to reach
    // the party/bag screens without needing to find a Pokemon Center.
    for _ in 0..10u32 {
        let mut s2 = |gb: &mut Gameboy, n: u32| sweep(gb, n);
        press(&mut gb, KeypadKey::Start, 6, 20, &mut s2);
        press(&mut gb, KeypadKey::A, 6, 20, &mut s2);
    }
    for _ in 0..30u32 {
        let mut s2 = |gb: &mut Gameboy, n: u32| sweep(gb, n);
        press(&mut gb, KeypadKey::Down, 4, 8, &mut s2);
        press(&mut gb, KeypadKey::A, 4, 12, &mut s2);
        press(&mut gb, KeypadKey::B, 4, 12, &mut s2);
    }

    let elapsed = start.elapsed();
    eprintln!(
        "ran {frame_no} frames in {:.2}s ({} bytes with at least one observed change, {} co-occurrence frame-events)",
        elapsed.as_secs_f64(),
        history.len(),
        co_occurrence.len()
    );

    // Write compact JSON: per-address change_frames + values_seen, plus the
    // co-occurrence log (frame -> which addresses moved together).
    use std::io::Write;
    let mut out = std::fs::File::create(&out_path).unwrap();
    write!(out, "{{\"total_frames\": {frame_no}, \"addresses\": {{").unwrap();
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
