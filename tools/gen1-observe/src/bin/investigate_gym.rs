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
    (0xCFE6, "wEnemyMonHP.hi"),
    (0xCFE7, "wEnemyMonHP.lo"),
    (0xD015, "wBattleMonHP.hi"),
    (0xD016, "wBattleMonHP.lo"),
    (0xCC26, "wCurrentMenuItem"),
    (0xCC34, "wMenuJoypadPollCount"),
    (0xCD6B, "wJoyIgnore"),
    (0xD125, "wTextBoxID"),
    // Round nine: hand-off item 4 is answered (a real gym battle plays to a
    // black-out) -- these are the move-mechanic addresses that same real
    // battle exercises but nothing has watched yet: which move each side
    // picked, its stats as loaded from the move data table, whether it
    // missed, and PP. wPlayerMoveListIndex/wEnemyMoveListIndex are already
    // described; these are their still-undescribed siblings.
    (0xCCEE, "wPlayerDisabledMoveNumber"),
    (0xCCEF, "wEnemyDisabledMoveNumber"),
    (0xCCF1, "wPlayerUsedMove"),
    (0xCCF2, "wEnemyUsedMove"),
    (0xCCF4, "wMoveDidntMiss"),
    (0xCFCC, "wEnemyMoveNum"),
    (0xCFCD, "wEnemyMoveEffect"),
    (0xCFCE, "wEnemyMovePower"),
    (0xCFCF, "wEnemyMoveType"),
    (0xCFD0, "wEnemyMoveAccuracy"),
    (0xCFD1, "wEnemyMoveMaxPP"),
    (0xCFD2, "wPlayerMoveNum"),
    (0xCFD3, "wPlayerMoveEffect"),
    (0xCFD4, "wPlayerMovePower"),
    (0xCFD5, "wPlayerMoveType"),
    (0xCFD6, "wPlayerMoveAccuracy"),
    (0xCFD7, "wPlayerMoveMaxPP"),
];

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let rom = std::fs::read(&rom_path).expect("read rom");
    // Round eight found that every earlier round's own coverage numbers were
    // produced with POKE_SAVE set (a real, already-progressed save reaches
    // far more of the game than a from-scratch new game does) -- this
    // session's own investigate_* binaries had been passing None instead,
    // which is a second, larger reason a forced battle might behave
    // differently from what main.rs's own production runs saw. Same staging
    // pattern main.rs already uses (Gameboy::new wants a path alongside the
    // ROM bytes when a save is present, not just the save bytes themselves).
    let mut gb = match save_path {
        Some(ref p) => {
            let staged_rom = "target/gym-staged.gb";
            let staged_sav = "target/gym-staged.sav";
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
    // `wCurrentMenuItem` sat at 1 for the whole rest of a pure A-mash run
    // (round six's first attempt) — plausibly PKMN, not FIGHT, on the main
    // battle menu's 4 items, if a pure-A mash never actually moves the
    // cursor off wherever the battle's own setup leaves it. Try explicitly:
    // B (back out of whatever is open), UP (toward FIGHT), A (pick it), A
    // (pick the first move, `MoveSelectionMenu`'s own "+1" indexing) —
    // rather than another longer blind mash of the same single button.
    let mut press = |gb: &mut Gameboy, key: KeypadKey, held: u32, gap: u32, frame: &mut u32| {
        gb.keydown(key);
        for _ in 0..held {
            poll(gb, frame);
        }
        gb.keyup(key);
        for _ in 0..gap {
            poll(gb, frame);
        }
    };
    for _ in 0..200u32 {
        press(&mut gb, KeypadKey::B, 6, 16, &mut frame);
        press(&mut gb, KeypadKey::Up, 6, 16, &mut frame);
        press(&mut gb, KeypadKey::A, 6, 16, &mut frame);
        press(&mut gb, KeypadKey::A, 6, 16, &mut frame);
    }
    eprintln!("done at frame {frame}");
}
