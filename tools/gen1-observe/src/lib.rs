// The deterministic script both passes replay, in one place. Pass 1 and pass
// 2 diverge only in HOW they advance one frame (`on_frame`); if they diverge
// in WHAT they do to the game, their frame numbers stop lining up and pass
// 2's targeting (built from pass 1's own output) silently traces the wrong
// frames. One shared driver is what keeps that impossible instead of merely
// unlikely.
use terminalgb::gameboy::Gameboy;
use terminalgb::KeypadKey;

pub type OnFrame<'a> = dyn FnMut(&mut Gameboy, u32) + 'a;

struct Driver<'a> {
    gb: &'a mut Gameboy,
    on_frame: &'a mut OnFrame<'a>,
    frame_no: u32,
}

impl<'a> Driver<'a> {
    fn tick(&mut self) {
        (self.on_frame)(self.gb, self.frame_no);
        self.frame_no += 1;
    }

    fn idle(&mut self, frames: u32) {
        for _ in 0..frames {
            self.tick();
        }
    }

    fn press(&mut self, key: KeypadKey, held: u32, gap: u32) {
        self.gb.keydown(key);
        for _ in 0..held {
            self.tick();
        }
        self.gb.keyup(key);
        for _ in 0..gap {
            self.tick();
        }
    }

    /// Cycle the cursor through `n` menu positions, trying (A) then backing
    /// out (B) at each one, moving DOWN between tries. Deliberately not
    /// precise about which item lands where — a menu's exact layout is
    /// exactly the kind of fact this tool exists to stop guessing at, so
    /// instead of guessing an order, visit every position in one pass. Two
    /// side effects this accepts: opening something the B-back-out does not
    /// fully close (rare, and the next DOWN/A still probes the next item
    /// from wherever that leaves the cursor) and a fully-closed menu ending
    /// the loop early (the DOWN presses become no-ops, which costs nothing).
    fn probe_menu(&mut self, n: u32, settle: u32, back_out_presses: u32) {
        for _ in 0..n {
            self.press(KeypadKey::A, 6, settle);
            for _ in 0..back_out_presses {
                self.press(KeypadKey::B, 6, 10);
            }
            self.press(KeypadKey::Down, 4, 8);
        }
    }

    /// Like `probe_menu`, but scrolls inside whatever opens rather than just
    /// glancing at it and backing straight out — for a list-shaped screen
    /// (the Pokédex, the bag) a plain probe only ever touches its first
    /// entry. Same "visit every position, don't guess the order" shape as
    /// `probe_menu`; the only difference is a scroll burst before backing
    /// out of each one.
    fn scroll_probe_menu(&mut self, n: u32, open_settle: u32, scroll_reps: u32,
                          back_out_presses: u32) {
        for _ in 0..n {
            self.press(KeypadKey::A, 6, open_settle);
            for _ in 0..scroll_reps {
                self.press(KeypadKey::Down, 4, 10);
            }
            for _ in 0..scroll_reps {
                self.press(KeypadKey::Up, 4, 10);
            }
            for _ in 0..back_out_presses {
                self.press(KeypadKey::B, 6, 10);
            }
            self.press(KeypadKey::Down, 4, 8);
        }
    }
}

pub fn run_script(gb: &mut Gameboy, on_frame: &mut OnFrame<'_>) {
    let mut d = Driver {
        gb,
        on_frame,
        frame_no: 0,
    };

    // --- Phase 1: title -> continue. Same shape as TerminalGB's own
    // gen1atlas.rs Tier C script.
    for frame in 0..1400u32 {
        let want = if (frame / 24) % 2 == 0 {
            KeypadKey::A
        } else {
            KeypadKey::Start
        };
        d.gb.keydown(want);
        d.tick();
        d.gb.keyup(want);
    }

    // --- Phase 2: walk and open menus, reaching the overworld with control.
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
        d.gb.keydown(want);
        d.tick();
        d.gb.keyup(want);
    }

    // --- Phase 3: stock the bag directly — the same "write an
    // already-verified address" technique battle-forcing already uses, not
    // a new kind of trick. wNumBagItems $D31D, wBagItems $D31E, both fully
    // described, fully evidenced entries in this atlas already. Item id 4
    // (POKé BALL) twice: it is both a plausible in-battle item to probe with
    // and, thrown at a wild Pokémon, a chance to exercise the catch/add-to-
    // party path along the way, not only the plain item-use path.
    d.gb.debug_write(0xD31D, 2); // wNumBagItems = 2
    d.gb.debug_write(0xD31E, 4); // slot 0: item 4 (POKé BALL)
    d.gb.debug_write(0xD31F, 5); // qty 5
    d.gb.debug_write(0xD320, 4); // slot 1: item 4 again
    d.gb.debug_write(0xD321, 5); // qty 5
    d.gb.debug_write(0xD322, 0xFF); // terminator
    d.idle(10);

    // --- Phase 4: a wild battle, forced the way the game's own debug menu
    // does — wCurOpponent $D059 and wCurEnemyLevel $D127, both already
    // fully evidenced, fully described entries. Boost experience first
    // (wPartyMon1Exp $D179, 3 bytes big-endian, also already described) so a
    // win has a real chance of crossing a level boundary — this does not
    // guarantee a level-up or an evolution, only raises the odds one
    // happens somewhere in this run; whether it actually did is measured
    // afterward, not assumed here.
    d.gb.debug_write(0xD179, 0x00);
    d.gb.debug_write(0xD17A, 0x13);
    d.gb.debug_write(0xD17B, 0x88); // wPartyMon1Exp = 0x001388 = 5000
    d.gb.debug_write(0xD059, 1); // wCurOpponent: species internal index 1
    d.gb.debug_write(0xD127, 5); // wCurEnemyLevel
    d.idle(30);
    // Turn 1: try the battle's ITEM option (DOWN from FIGHT reaches it on
    // the 2x2 FIGHT/PKMN/ITEM/RUN menu), then its first bag slot — probing
    // the in-battle item-use path once, deliberately, rather than only
    // attacking. If nothing is actually open at that moment the presses are
    // harmless no-ops.
    d.press(KeypadKey::Down, 6, 16);
    d.press(KeypadKey::A, 6, 16);
    d.press(KeypadKey::A, 6, 16);
    d.press(KeypadKey::B, 6, 16);
    // The rest of the battle: mash A, which selects FIGHT then the first
    // move on the menus that follow, and advances any text box in between.
    for _ in 0..40u32 {
        d.press(KeypadKey::A, 8, 16);
    }

    // --- Phase 5: a trainer battle. Class 6 (JR.TRAINER♀), roster 1 —
    // GOLDEEN, level 19 — is a fact this atlas already carries, measured
    // and written up in cerulean-gym.md, not a new guess. wCurOpponent =
    // 200 + class, per the same debug-menu technique.
    d.gb.debug_write(0xD059, 206);
    d.gb.debug_write(0xD05D, 1); // wTrainerNo: roster 1
    d.idle(30);
    for _ in 0..40u32 {
        d.press(KeypadKey::A, 8, 16);
    }

    // --- Phase 6: force the party down to the edge of fainting, then force
    // a battle against a much stronger wild opponent — reaching the faint /
    // black-out path deliberately rather than hoping a battle happens to go
    // badly. wPartyMon1HP $D16C, 2 bytes big-endian, already described.
    d.gb.debug_write(0xD16C, 0x00);
    d.gb.debug_write(0xD16D, 0x01); // wPartyMon1HP = 1
    d.gb.debug_write(0xD059, 1);
    d.gb.debug_write(0xD127, 60); // a level far above what one hit needs
    d.idle(30);
    for _ in 0..30u32 {
        d.press(KeypadKey::A, 8, 16);
    }
    // Whatever the black-out sequence needs beyond that (it is a cutscene,
    // not a menu) — just keep advancing text/prompts.
    for _ in 0..20u32 {
        d.press(KeypadKey::A, 6, 20);
    }

    // --- Phase 7: the START menu, probed position by position rather than
    // navigated by a guessed layout — reaches PARTY, ITEM, the trainer
    // card, SAVE, OPTIONS and EXIT across the pass, in whichever order this
    // cartridge's menu actually lists them.
    d.press(KeypadKey::Start, 6, 20);
    d.probe_menu(7, 20, 2);
    d.press(KeypadKey::B, 6, 10);

    // --- Phase 8: inside the party list specifically (re-entering by
    // probing again, since phase 7 already backed all the way out), try to
    // reorder: select the first slot, take whatever sub-menu comes up,
    // probe it, then the second slot.
    d.press(KeypadKey::Start, 6, 20);
    d.press(KeypadKey::A, 6, 20); // first START item — POKEMON on this cartridge
    d.press(KeypadKey::A, 6, 20); // party slot 1
    d.probe_menu(4, 16, 1); // whatever slot-1 submenu that opens (STATS/SWITCH/CANCEL, or similar)
    d.press(KeypadKey::Down, 4, 8);
    d.press(KeypadKey::A, 6, 20); // party slot 2, completing a SWITCH if one was armed
    d.press(KeypadKey::B, 6, 10);
    d.press(KeypadKey::B, 6, 10);

    // --- Phase 9: save. Cycling the START menu again and pressing A+A at
    // each position already exercises whichever position SAVE is at from
    // phase 7, but do it once more, deliberately, with more settle time —
    // saving is exactly the write-heavy moment (the whole `save` chapter)
    // worth a dedicated, unhurried attempt rather than one probe among
    // several.
    d.press(KeypadKey::Start, 6, 20);
    d.probe_menu(7, 30, 1);
    d.press(KeypadKey::B, 6, 10);

    // --- Phase 10: back in the overworld, walk and open menus again — the
    // same shape as phase 2 — so anything phases 3-9 left mid-transition
    // has room to settle, and so the run ends on ordinary play rather than
    // mid-battle-or-menu state.
    for frame in 0..1200u32 {
        let want = match (frame / 20) % 8 {
            0 | 1 => KeypadKey::Down,
            2 => KeypadKey::A,
            3 => KeypadKey::Up,
            4 => KeypadKey::Left,
            5 => KeypadKey::Start,
            6 => KeypadKey::Right,
            _ => KeypadKey::B,
        };
        d.gb.keydown(want);
        d.tick();
        d.gb.keyup(want);
    }

    // --- Phase 11: deeper menu exploration — the Pokédex and the bag,
    // scrolled rather than glanced at. Phases 7 and 9 already open every
    // START-menu position once each, but `probe_menu` only ever touches a
    // list screen's first entry before backing out; a Pokédex with more
    // than one species seen, or a bag with more than one item, needs actual
    // scrolling to reach entry 2 and beyond. Appended after phase 10 rather
    // than inserted earlier so every existing phase's frame numbers stay
    // exactly what they were — this only adds frames, it never moves any.
    d.press(KeypadKey::Start, 6, 20);
    d.scroll_probe_menu(7, 20, 6, 2);
    d.press(KeypadKey::B, 6, 10);

    // --- Phase 12: back to ordinary play, the same shape as phases 2 and
    // 10, so the run still ends on ordinary overworld play rather than
    // mid-menu.
    for frame in 0..800u32 {
        let want = match (frame / 20) % 8 {
            0 | 1 => KeypadKey::Down,
            2 => KeypadKey::A,
            3 => KeypadKey::Up,
            4 => KeypadKey::Left,
            5 => KeypadKey::Start,
            6 => KeypadKey::Right,
            _ => KeypadKey::B,
        };
        d.gb.keydown(want);
        d.tick();
        d.gb.keyup(want);
    }
}
