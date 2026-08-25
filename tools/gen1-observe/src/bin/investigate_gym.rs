// Ranked hand-off item 4 (docs/observation.md): a full gym battle (Misty,
// not a JR.TRAINER), forced to a clear loss rather than a low-HP battle that
// could resolve either way — and, per round five's own lesson about the
// wild-battle phase never actually starting, checked for whether the forced
// trainer battle reliably starts at all before assuming it does.
//
// First attempt (not this version — see docs/observation.md) forced the
// battle right after the *full* shared script (`run_script`) returned, and
// found the forced writes clobbered within 0-2 frames regardless of how long
// it idled first: something left running by the script's own final phases
// (party-reorder, a save attempt, the earlier forced blackout) keeps
// touching that WRAM region even with no input at all. This version forces
// the battle in the same *kind* of state phase 5's already-successful
// trainer-battle forcing used — right after intro and one walk phase, before
// any menu/save/blackout activity — by duplicating just those two phases
// inline rather than editing the shared script (which would move every
// existing pass's frame numbers).
use terminalgb::gameboy::Gameboy;
use terminalgb::KeypadKey;

const WATCH: &[(u16, &str)] = &[
    (0xD057, "wIsInBattle"),
    (0xD059, "wCurOpponent"),
    (0xD031, "wTrainerClass"),
    (0xCFE5, "wEnemyMonSpecies"),
    (0xD8A4, "wEnemyMon1Species"),
    (0xD16C, "wPartyMon1HP.hi"),
    (0xD16D, "wPartyMon1HP.lo"),
    (0xD356, "wObtainedBadges"),
];

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let rom = std::fs::read(&rom_path).expect("read rom");
    let mut gb = Gameboy::new(rom, None);

    // Phase 1: title -> continue, identical to lib.rs's run_script.
    for frame in 0..1400u32 {
        let want = if (frame / 24) % 2 == 0 {
            KeypadKey::A
        } else {
            KeypadKey::Start
        };
        gb.keydown(want);
        gb.frame();
        gb.keyup(want);
    }
    // Phase 2: walk and open menus, identical to lib.rs's run_script.
    for frame in 0..1400u32 {
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
        gb.frame();
        gb.keyup(want);
    }
    eprintln!("intro + one walk phase done; forcing the Misty battle now");

    // wPartyMon1HP = 1 -- the same already-described, already-evidenced
    // technique phase 6 of run_script itself uses, so a single hit ends it.
    gb.debug_write(0xD16C, 0x00);
    gb.debug_write(0xD16D, 0x01);
    // wCurOpponent = 235 (200 + class 35, MISTY); wTrainerNo = 1 (her only
    // roster entry, same convention as phase 5's JR.TRAINER forcing).
    gb.debug_write(0xD059, 235);
    gb.debug_write(0xD05D, 1);

    let mut last: Vec<u8> = WATCH.iter().map(|(a, _)| gb.peek(*a)).collect();
    println!("frame\taddress\tname\told\tnew");
    let mut frame = 0u32;
    let mut poll = |gb: &mut Gameboy, frame: &mut u32| {
        gb.frame();
        for (i, (addr, name)) in WATCH.iter().enumerate() {
            let v = gb.peek(*addr);
            if v != last[i] {
                println!("{frame}\t${addr:04X}\t{name}\t{}\t{}", last[i], v);
                last[i] = v;
            }
        }
        *frame += 1;
    };
    for _ in 0..30u32 {
        poll(&mut gb, &mut frame);
    }
    for _ in 0..400u32 {
        gb.keydown(KeypadKey::A);
        for _ in 0..8 {
            poll(&mut gb, &mut frame);
        }
        gb.keyup(KeypadKey::A);
        for _ in 0..16 {
            poll(&mut gb, &mut frame);
        }
    }
    eprintln!("done at frame {frame}");
}
