# Parties manquantes — drop-in

À copier dans le clone `kognitara/amentys`. Rien de tout ça ne touche `re` ni le driver NVMe.

## 1. plan

```
cp plan/ops.rs   <repo>/plan/src/ops.rs
cp plan/sceau.rs <repo>/plan/src/sceau.rs
```

Dans `plan/src/lib.rs`, en haut :

```rust
pub mod layer;
pub mod ops;
pub mod sceau;
```

Dans `plan/Cargo.toml`, sous `[dependencies]` :

```toml
blake3 = { workspace = true }
```

`ops.rs` a besoin de `blake3` (le workspace l'a déjà). `Noun::from_bytes` / `Noun::of` / `as_bytes` sont déjà dans `noun`.

## 2. ocean

```
cp ocean/lib.rs <repo>/ocean/src/lib.rs
```

Remplace le `Vec<Noun, 256>`. L'alias `Ocean = CoreOcean` garde `maat` compilable.  
`put` / `get` = working set. Le NVMe (`ra`) devient DiskOcean plus tard, même signatures.

## 3. maat

```
cp maat/law.rs <repo>/maat/src/law.rs
```

Dans `maat/src/lib.rs` :

```rust
pub mod law;
```

`maat` doit déjà dépendre de `plan`. Si `sceau` n'est pas visible : `plan = { workspace = true }` dans `maat/Cargo.toml`.

Dans `maat/src/main.rs`, **après** `Plan::new(...)`, avant Prism :

```rust
let plan = terminal_plan.expect("plan");
let sceau = plan::sceau::Sceau::birth(&plan, 1_000);
match maat::law::weigh(&sceau, &plan) {
    plan::sceau::Verdict::Accept => {}
    plan::sceau::Verdict::Refuse(why) => panic!("maat refused: {why}"),
}
```

Interdiction : `Noun::new([0x01; 32])`. Partout : `Noun::of(b"tui")` (ou le blob réel).

## 4. ji fantôme

Dans le `Cargo.toml` racine, **retire** :

```toml
ji = { path = "ji" }
```

des `workspace.dependencies` — le dossier n'est pas dans `members`.

## 5. Vérif

```
cargo test -p noun -p ocean -p plan
```

(host, pas `x86_64-amentys`). Les tests de `ops` et `sceau` passent en `std`.

Ce que ce patch ne fait **pas** : DiskOcean sur NVMe, Face, défenseurs, Pulse-clés. C'est le cran d'après, une fois `put`/`get` + `weigh` vus dans QEMU.
