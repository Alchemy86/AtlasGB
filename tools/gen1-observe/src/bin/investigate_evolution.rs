// Round ten: a first, well-scoped probe at evolution, one of the captain's
// named coverage gaps (shops, evolution, learning/forgetting moves, the
// Pokedex, fishing, HMs). wForceEvolution ($CCD4) is an undescribed,
// rom,live-tier entry whose name is suggestive of a debug/test hook the
// same way `wForceEvolution` reads -- but per this project's own standing
// rule, a symbol name is not a source, so this writes a plausible non-zero
// value into it from a stable overworld state (the same kind of state
// investigate_gym's own successful forcing starts from) and WATCHES,
// exactly the same "force it, then check before trusting it" shape that
// worked for the gym battle, rather than assuming the name's implication is
// correct.
use terminalgb::gameboy::Gameboy;
use terminalgb::KeypadKey;

const WATCH: &[(u16, &str)] = &[
    (0xCCD4, "wForceEvolution"),
    (0xD121, "wEvolutionOccurred"),
    (0xD16B, "wPartyMon1Species"),
    (0xD18C, "wPartyMon1Level"),
    (0xD16E, "wPartyMon1BoxLevel"),
    (0xD156, "wEvoStoneItemID"),
    (0xCF92, "wWhichPokemon"),
    (0xD125, "wTextBoxID"),
    (0xCC26, "wCurrentMenuItem"),
];

fn main() {
    let rom_path = std::env::var("POKE_ROM").expect("set POKE_ROM");
    let save_path = std::env::var("POKE_SAVE").ok();
    let rom = std::fs::read(&rom_path).expect("read rom");
    let mut gb = match save_path {
        Some(ref p) => {
            let staged_rom = "target/evo-staged.gb";
            let staged_sav = "target/evo-staged.sav";
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

    // Phases 1-2: identical to investigate_gym.rs's own duplicated setup --
    // intro -> continue, then one walk phase, so this starts from the same
    // kind of stable overworld state the gym-battle forcing already proved
    // reliable, before touching anything evolution-specific.
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
    eprintln!("intro + one walk phase done; forcing wForceEvolution now");

    let before_species = gb.peek(0xD16B);
    let before_level = gb.peek(0xD18C);
    gb.debug_write(0xCCD4, 1);

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
    // Idle, watching -- if some background/overworld-tick routine notices
    // the flag without needing a menu or battle to invoke it, this alone
    // would show it. 600 frames (10 real-time seconds) is generous given
    // investigate_gym's own forced writes resolved within a few hundred.
    for _ in 0..600u32 {
        poll(&mut gb, &mut frame);
    }
    eprintln!(
        "done at frame {frame}. wForceEvolution before={}, still={}. species {}->{}, level {}->{}",
        1,
        gb.peek(0xCCD4),
        before_species,
        gb.peek(0xD16B),
        before_level,
        gb.peek(0xD18C)
    );
}
