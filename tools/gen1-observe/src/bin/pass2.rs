// Pass 2: re-run the identical shared script (src/lib.rs), and for the
// specific frames pass 1 flagged as containing a target-address change,
// single-step instruction by instruction instead of calling frame()
// wholesale — replicating Gameboy::frame()'s own loop with the same public
// methods it uses internally, so behaviour is identical and only the
// observation is finer-grained. Records, per target address, every (frame,
// pc, bank, old_value, new_value) write event.
use gen1_observe::run_script;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use terminalgb::gameboy::Gameboy;

#[derive(serde::Deserialize)]
struct Pass2Input {
    frames: Vec<u32>,
    addrs: Vec<String>,
}

struct WriteEvent {
    frame: u32,
    pc: u16,
    bank: usize,
    old: u8,
    new: u8,
}

fn traced_frame(
    gb: &mut Gameboy,
    frame_no: u32,
    shadow: &mut BTreeMap<u16, u8>,
    target_addrs: &[u16],
    events: &mut BTreeMap<u16, Vec<WriteEvent>>,
    instr_co: &mut Vec<(u32, u16, Vec<u16>)>,
) {
    gb.begin_frame();
    loop {
        let pc = gb.debug_pc();
        let bank = gb.debug_rom_bank();
        gb.step_instruction();
        let mut changed_now = Vec::new();
        for a in target_addrs {
            let now = gb.peek(*a);
            let before = shadow[a];
            if now != before {
                events.entry(*a).or_default().push(WriteEvent {
                    frame: frame_no,
                    pc,
                    bank,
                    old: before,
                    new: now,
                });
                shadow.insert(*a, now);
                changed_now.push(*a);
            }
        }
        if changed_now.len() > 1 {
            instr_co.push((frame_no, pc, changed_now));
        }
        if gb.check_and_reset_gpu_updated() {
            break;
        }
    }
    gb.end_frame();
}

#[allow(clippy::too_many_arguments)]
fn do_frame(
    gb: &mut Gameboy,
    frame_no: u32,
    target_frames: &BTreeSet<u32>,
    shadow: &mut BTreeMap<u16, u8>,
    target_addrs: &[u16],
    events: &mut BTreeMap<u16, Vec<WriteEvent>>,
    instr_co: &mut Vec<(u32, u16, Vec<u16>)>,
) {
    if target_frames.contains(&frame_no) {
        traced_frame(gb, frame_no, shadow, target_addrs, events, instr_co);
    } else {
        gb.frame();
        for a in target_addrs {
            let now = gb.peek(*a);
            if now != shadow[a] {
                events.entry(*a).or_default().push(WriteEvent {
                    frame: frame_no,
                    pc: 0xFFFF,
                    bank: 0,
                    old: shadow[a],
                    new: now,
                });
                shadow.insert(*a, now);
            }
        }
    }
}

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let input_path =
        std::env::var("PASS2_INPUT").unwrap_or_else(|_| "/tmp/pass2-input.json".into());
    let out_path = std::env::var("OBSERVE_OUT").unwrap_or_else(|_| "observe-pass2.json".into());

    let input: Pass2Input =
        serde_json::from_reader(std::fs::File::open(&input_path).unwrap()).unwrap();
    let target_frames: BTreeSet<u32> = input.frames.iter().copied().collect();
    let target_addrs: Vec<u16> = input
        .addrs
        .iter()
        .map(|s| u16::from_str_radix(s, 16).unwrap())
        .collect();

    let data = std::fs::read(&rom_path).expect("read rom");
    let mut gb = match save_path {
        Some(ref p) => {
            let staged_rom = "target/observe-staged2.gb";
            let staged_sav = "target/observe-staged2.sav";
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

    let mut events: BTreeMap<u16, Vec<WriteEvent>> = BTreeMap::new();
    let mut instr_co: Vec<(u32, u16, Vec<u16>)> = Vec::new();
    let mut shadow: BTreeMap<u16, u8> = target_addrs.iter().map(|a| (*a, gb.peek(*a))).collect();

    {
        let mut on_frame = |gb: &mut Gameboy, frame_no: u32| {
            do_frame(
                gb,
                frame_no,
                &target_frames,
                &mut shadow,
                &target_addrs,
                &mut events,
                &mut instr_co,
            );
        };
        run_script(&mut gb, &mut on_frame);
    }

    eprintln!(
        "pass2: {} target frames requested, {} addresses with at least one write event, {} multi-address instruction co-occurrences",
        target_frames.len(),
        events.len(),
        instr_co.len()
    );

    let mut out = std::fs::File::create(&out_path).unwrap();
    write!(out, "{{\"events\": {{").unwrap();
    let mut first = true;
    for (addr, evs) in &events {
        if !first {
            write!(out, ",").unwrap();
        }
        first = false;
        write!(out, "\"{addr:04X}\": [").unwrap();
        for (i, e) in evs.iter().enumerate() {
            if i > 0 {
                write!(out, ",").unwrap();
            }
            write!(
                out,
                "{{\"frame\": {}, \"pc\": \"{:04X}\", \"bank\": {}, \"old\": {}, \"new\": {}}}",
                e.frame, e.pc, e.bank, e.old, e.new
            )
            .unwrap();
        }
        write!(out, "]").unwrap();
    }
    write!(out, "}}, \"instr_co_occurrence\": [").unwrap();
    for (i, (f, pc, addrs)) in instr_co.iter().enumerate() {
        if i > 0 {
            write!(out, ",").unwrap();
        }
        let a: Vec<String> = addrs.iter().map(|x| format!("\"{x:04X}\"")).collect();
        write!(out, "[{f}, \"{pc:04X}\", [{}]]", a.join(",")).unwrap();
    }
    writeln!(out, "]}}").unwrap();
    eprintln!("wrote {out_path}");
}
