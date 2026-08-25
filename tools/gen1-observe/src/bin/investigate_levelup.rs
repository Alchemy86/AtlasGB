// Ranked hand-off item 3 (docs/observation.md): round three forced
// wPartyMon1Exp to 5000 before the wild battle to raise the odds of a
// level-up, and measured that neither wPartyMon1Level nor wPartyMon1Species
// ever changed across the whole run. This traces the handful of addresses
// that would explain why, frame by frame, instead of guessing from the
// pokered source (no checkout is available in this environment) — the same
// "watch it happen" standard as the rest of this tool.
use gen1_observe::run_script;
use terminalgb::gameboy::Gameboy;

const WATCH: &[(u16, &str)] = &[
    (0xD057, "wIsInBattle"),
    (0xD059, "wCurOpponent"),
    (0xD127, "wCurEnemyLevel"),
    (0xD16B, "wPartyMon1Species"),
    (0xD16C, "wPartyMon1HP.hi"),
    (0xD16D, "wPartyMon1HP.lo"),
    (0xD179, "wPartyMon1Exp.hi"),
    (0xD17A, "wPartyMon1Exp.mid"),
    (0xD17B, "wPartyMon1Exp.lo"),
    (0xD18C, "wPartyMon1Level"),
];

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let rom = std::fs::read(&rom_path).expect("read rom");
    // Round eight found every earlier round's real coverage numbers came
    // from runs with POKE_SAVE set, and that a from-scratch new game (this
    // binary's original None here) behaves very differently in a forced
    // battle -- so this round's own original finding (phase 5's trainer
    // battle apparently never concluding) needs re-checking under the same
    // save-loaded conditions production runs actually use, not left standing
    // on a run that turned out not to match them. Same staging pattern
    // main.rs already uses.
    let mut gb = match save_path {
        Some(ref p) => {
            let staged_rom = "target/levelup-staged.gb";
            let staged_sav = "target/levelup-staged.sav";
            std::fs::create_dir_all("target").ok();
            std::fs::copy(&rom_path, staged_rom).unwrap();
            std::fs::copy(p, staged_sav).unwrap();
            Gameboy::new(
                std::fs::read(staged_rom).unwrap(),
                Some(std::path::PathBuf::from(staged_rom)),
            )
        }
        None => Gameboy::new(rom, None),
    };

    let mut last: Vec<u8> = WATCH.iter().map(|(a, _)| gb.peek(*a)).collect();
    println!("frame\taddress\tname\told\tnew");
    let mut on_frame = |gb: &mut Gameboy, frame: u32| {
        gb.frame();
        for (i, (addr, name)) in WATCH.iter().enumerate() {
            let v = gb.peek(*addr);
            if v != last[i] {
                println!("{frame}\t${addr:04X}\t{name}\t{}\t{}", last[i], v);
                last[i] = v;
            }
        }
    };
    run_script(&mut gb, &mut on_frame);
}
