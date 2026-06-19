# Game Data Licensing Decision (P1-1)

**Date:** 2026-06-19

## Sources

- **SR4:** ChummerGen for SR4 — cloned into `vendor/chummer-sr4/`; data in `bin/data/*.xml`
- **SR5:** Chummer5a — cloned into `vendor/chummer5a/`; data in `Chummer/data/*.xml`

## License stance

Both Chummer tools are open source (GPL). However, the XML data files encode copyrighted
Shadowrun content owned by Catalyst Game Labs. We do **not** redistribute either the raw
XML or the derived `game_data.db`.

Decision confirmed by the project owner: use locally for tool development only.

Enforcement:
- `vendor/` is git-ignored (was already true from project start)
- `game_data.db` added to `.gitignore` in this commit

## Migration result

Running `make migrate` (`cargo run --bin personafix-migrate`) against both vendor repos:

```
SR5: 63 books, 5 metatypes, 76 skills, 803 qualities, 632 weapons, 202 armor,
     575 augmentations, 363 spells
SR4: 42 books, 5 metatypes, 78 skills, 483 qualities, 743 weapons, 192 armor,
     439 augmentations, 253 spells
```

`game_data.db` lives at the project root (git-ignored). To regenerate:

```sh
make migrate
```

## Follow-up

P1-2 is done: `game_data.db` exists and the L3a smoke test can be un-ignored.
Next: P3-1 (wire skill/quality panels to real data), P3-2 (augmentations), P3-5 (magic).
