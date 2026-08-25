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
    let rom = std::fs::read(&rom_path).expect("read rom");
    let mut gb = Gameboy::new(rom, None);

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
