// A one-question trace, same shape as investigate_gym.rs and
// investigate_evolution.rs: does the cartridge actually hold `NINTEN`/`SONY`
// in wPlayerName/wRivalName as the DEFAULT before the naming screens run, not
// merely somewhere in the ROM?
//
// Deliberately boots with NO save (round eight's own lesson — a real save
// reaches more of the *midgame* than a fresh one — does not apply here: the
// thing under test only exists in the first few seconds of a brand-new game,
// before any save file's player has already been named, so a fresh boot is
// the only state that can even ask the question).
//
// Mechanism under test: `PrepareOakSpeech` unconditionally copies two 11-byte
// debug names into wPlayerName/wRivalName before `OakSpeech` decides whether
// to run the real naming screens (`ChoosePlayerName`/`ChooseRivalName`) or
// skip them — the skip is gated on wStatusFlags6 ($D732) bit 1
// (BIT_DEBUG_MODE), which `StartNewGame` explicitly clears on every ordinary
// new game. So on an unmodified cartridge the debug names are always written,
// then always immediately overwritten by the real naming screens. Forcing
// that one bit set — the same debug_write-a-known-flag technique
// investigate_gym.rs already uses for wCurOpponent — reaches the same
// `.skipSpeech` branch a ROM hack that deleted the calls to
// ChoosePlayerName/ChooseRivalName would reach, without editing a single ROM
// byte. Forced every frame (not once) because StartNewGame's own `res` runs
// at an unknown frame during the button-mash below; a one-shot force could
// land before that reset and be silently undone.
//
// DEFAULTNAMES_FORCE_DEBUG=0 runs the same script WITHOUT forcing the bit, as
// a control: if the claim's mechanism is right, that run should NOT end with
// wPlayerName/wRivalName reading NINTEN/SONY, because the real naming screens
// run and overwrite them (or the button mash simply never completes a name,
// leaving something other than the debug default).
use terminalgb::gameboy::Gameboy;
use terminalgb::KeypadKey;

const W_STATUS_FLAGS_6: u16 = 0xD732;
const BIT_DEBUG_MODE: u8 = 0b0000_0010; // bit 1, per pokered's wram.asm const_def
const W_PLAYER_NAME: u16 = 0xD158;
const W_RIVAL_NAME: u16 = 0xD34A;
const NAME_LENGTH: usize = 11;

/// Decode Gen 1 text: 'A'-'Z' are $80-$99, $50 terminates. Anything else
/// (padding, or bytes copied past a short source string's own terminator —
/// see the header comment on why an 11-byte copy of a 7-byte "NINTEN@" reads
/// past its own end) is rendered as a hex escape rather than guessed at.
fn decode_name(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &b in bytes {
        match b {
            0x50 => {
                out.push_str("[END]");
                break;
            }
            0x80..=0x99 => out.push((b'A' + (b - 0x80)) as char),
            other => out.push_str(&format!("<{other:02X}>")),
        }
    }
    out
}

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let force_debug = std::env::var("DEFAULTNAMES_FORCE_DEBUG")
        .map(|v| v != "0")
        .unwrap_or(true);
    let rom = std::fs::read(&rom_path).expect("read rom");

    // No POKE_SAVE staging here, deliberately — see header comment.
    let mut gb = Gameboy::new(rom, None);

    println!("force_debug={force_debug}");
    println!("frame\taddress\tname\told\tnew");

    let mut last_player = gb.peek_range(W_PLAYER_NAME, NAME_LENGTH);
    let mut last_rival = gb.peek_range(W_RIVAL_NAME, NAME_LENGTH);
    let mut frame = 0u32;

    let mut poll = |gb: &mut Gameboy, frame: &mut u32| {
        if force_debug {
            let cur = gb.peek(W_STATUS_FLAGS_6);
            gb.debug_write(W_STATUS_FLAGS_6, cur | BIT_DEBUG_MODE);
        }
        gb.frame();
        let player = gb.peek_range(W_PLAYER_NAME, NAME_LENGTH);
        if player != last_player {
            println!(
                "{frame}\t${W_PLAYER_NAME:04X}\twPlayerName\t{}\t{}",
                decode_name(&last_player),
                decode_name(&player)
            );
            last_player = player;
        }
        let rival = gb.peek_range(W_RIVAL_NAME, NAME_LENGTH);
        if rival != last_rival {
            println!(
                "{frame}\t${W_RIVAL_NAME:04X}\twRivalName\t{}\t{}",
                decode_name(&last_rival),
                decode_name(&rival)
            );
            last_rival = rival;
        }
        *frame += 1;
    };

    // Phase 1: title -> new game. Identical alternating-A/Start mash lib.rs's
    // run_script uses to reach CONTINUE on an existing save; with no save
    // present the equivalent main-menu item is NEW GAME (the only one besides
    // OPTIONS), so the same mash is expected to reach it the same way.
    for f in 0..1400u32 {
        let want = if (f / 24) % 2 == 0 {
            KeypadKey::A
        } else {
            KeypadKey::Start
        };
        gb.keydown(want);
        poll(&mut gb, &mut frame);
        gb.keyup(want);
    }
    eprintln!("phase 1 (title -> new game) done at frame {frame}");

    // Phase 2: OakSpeech itself is a long, mostly non-interactive cutscene
    // (pics fading in, text boxes) with two real input gates when NOT
    // skipped — ChoosePlayerName and ChooseRivalName's naming screens. Mash
    // A/B/Down throughout: on the force_debug run this is irrelevant (the
    // .skipSpeech branch never reaches either naming screen); on the control
    // run it is the same kind of "advance whatever's open" mash phase 5/6 of
    // run_script already use for cutscenes and menus.
    for _ in 0..2600u32 {
        gb.keydown(KeypadKey::A);
        poll(&mut gb, &mut frame);
        gb.keyup(KeypadKey::A);
        gb.keydown(KeypadKey::Down);
        poll(&mut gb, &mut frame);
        gb.keyup(KeypadKey::Down);
    }
    eprintln!("phase 2 (OakSpeech window) done at frame {frame}");

    let final_player = gb.peek_range(W_PLAYER_NAME, NAME_LENGTH);
    let final_rival = gb.peek_range(W_RIVAL_NAME, NAME_LENGTH);
    println!(
        "FINAL\twPlayerName\t{}\t{:02X?}",
        decode_name(&final_player),
        final_player
    );
    println!(
        "FINAL\twRivalName\t{}\t{:02X?}",
        decode_name(&final_rival),
        final_rival
    );
}
