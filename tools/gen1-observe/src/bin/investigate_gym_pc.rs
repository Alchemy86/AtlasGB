// Round ten, hand-off item 1 from docs/observation.md: pass 2 (instruction-
// level) tracing over investigate_gym's own already-proven forced battle, to
// find out whether the twelve move-mechanic addresses round nine described
// from frame-level (pass 1) data share a writer -- which would earn them a
// real `related` entry, not the frame-co-occurrence guess this project has
// already tried once and dropped as too noisy.
//
// pass2.rs cannot be reused as-is: it is hardcoded to replay lib.rs's shared
// `run_script`, and this forced Misty battle is investigate_gym's own script,
// not run_script (investigate_gym.rs's own header comment explains why it
// duplicates phases 1-2 inline instead of using the shared one). So this
// binary duplicates investigate_gym's exact setup -- same phases, same
// forced writes, same B/Up/A/A navigation -- and replaces its frame-level
// `gb.frame()` polling with pass2.rs's own proven instruction-level loop
// (`begin_frame`/`step_instruction`/`check_and_reset_gpu_updated`/
// `end_frame`, the same public Gameboy API pass2.rs already uses). Frame
// numbers stay identical to investigate_gym's own run -- only the
// observation is finer-grained, the same relationship pass 1 and pass 2 have
// to each other project-wide.
use terminalgb::gameboy::Gameboy;
use terminalgb::KeypadKey;

const WATCH: &[(u16, &str)] = &[
    (0xD057, "wIsInBattle"),
    (0xCFE7, "wEnemyMonHP.lo"),
    (0xD016, "wBattleMonHP.lo"),
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
    // Round ten: still-undescribed `battle`-group addresses this same real
    // gym battle plausibly exercises (HP-bar animation, gym-leader/trainer
    // class bookkeeping, unmodified levels, damage multipliers) -- widening
    // the watch on an already-proven-reliable battle rather than guessing at
    // a new game state to force.
    (0xD05C, "wGymLeaderNo"),
    (0xCD2D, "wEngagedTrainerClass"),
    (0xD713, "wEnemyMonOrTrainerClass"),
    (0xCD0F, "wPlayerMonUnmodifiedLevel"),
    (0xCD23, "wEnemyMonUnmodifiedLevel"),
    (0xCCE3, "wLastSwitchInEnemyMonHP.hi"),
    (0xCCE4, "wLastSwitchInEnemyMonHP.lo"),
    (0xCEEB, "wHPBarOldHP"),
    (0xCEED, "wHPBarNewHP.hi"),
    (0xCEEE, "wHPBarNewHP.lo"),
    (0xCEEF, "wHPBarDelta"),
    (0xCEF0, "wHPBarTempHP[0]"),
    (0xCEFD, "wHPBarHPDifference[0]"),
    (0xD05B, "wDamageMultipliers"),
    (0xCCDB, "wMoveMenuType"),
    (0xCD6D, "wBattleMenuCurrentPP[0]"),
    (0xCD6E, "wBattleMenuCurrentPP[1]"),
    (0xCD6F, "wBattleMenuCurrentPP[2]"),
    (0xCD70, "wBattleMenuCurrentPP[3]"),
    (0xCCF6, "wLowHealthAlarmDisabled"),
    (0xCCF0, "wInHandlePlayerMonFainted"),
    (0xCC2D, "wBattleAndStartSavedMenuItem"),
    (0xCC79, "wAnimPalette[0]"),
    (0xCCD5, "wAILayer2Encouragement.hi"),
    (0xCCD6, "wAILayer2Encouragement.lo"),
    (0xCD47, "wBattleTransitionSpiralDirection"),
];

fn traced_frame(gb: &mut Gameboy, frame_no: u32, last: &mut [u8]) {
    gb.begin_frame();
    loop {
        let pc = gb.debug_pc();
        let bank = gb.debug_rom_bank();
        gb.step_instruction();
        for (i, (addr, name)) in WATCH.iter().enumerate() {
            let v = gb.peek(*addr);
            if v != last[i] {
                println!(
                    "{frame_no}\t${addr:04X}\t{name}\t{}\t{}\tbank{bank}:${pc:04X}",
                    last[i], v
                );
                last[i] = v;
            }
        }
        if gb.check_and_reset_gpu_updated() {
            break;
        }
    }
    gb.end_frame();
}

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let rom = std::fs::read(&rom_path).expect("read rom");
    let mut gb = match save_path {
        Some(ref p) => {
            let staged_rom = "target/gym-pc-staged.gb";
            let staged_sav = "target/gym-pc-staged.sav";
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

    // Phases 1-2: identical to investigate_gym.rs's own duplicated setup.
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
    eprintln!("intro + one walk phase done; forcing the Misty battle now (instruction-traced)");

    gb.debug_write(0xD16C, 0x00);
    gb.debug_write(0xD16D, 0x01);
    gb.debug_write(0xD059, 235);
    gb.debug_write(0xD05D, 1);

    let mut last: Vec<u8> = WATCH.iter().map(|(a, _)| gb.peek(*a)).collect();
    println!("frame\taddress\tname\told\tnew\twriter");
    let mut frame = 0u32;
    let mut poll = |gb: &mut Gameboy, frame: &mut u32| {
        traced_frame(gb, *frame, &mut last);
        *frame += 1;
    };
    for _ in 0..30u32 {
        poll(&mut gb, &mut frame);
    }
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
    // Stop once the battle ends in a black-out (wIsInBattle -> $FF), same
    // trigger round nine's own trace found at frame 3,179 -- no need to
    // instruction-trace the ~14,000 idle frames investigate_gym.rs's own
    // fixed press budget runs afterward.
    for _ in 0..200u32 {
        if gb.peek(0xD057) == 0xFF {
            break;
        }
        press(&mut gb, KeypadKey::B, 6, 16, &mut frame);
        press(&mut gb, KeypadKey::Up, 6, 16, &mut frame);
        press(&mut gb, KeypadKey::A, 6, 16, &mut frame);
        press(&mut gb, KeypadKey::A, 6, 16, &mut frame);
    }
    eprintln!("done at frame {frame}");
}
