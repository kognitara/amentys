# Amentys

```txt
                    ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
                    │     THOT      │   │     MAAT      │   │   AMENTYS     │
                    │ King of Truth │   │ Queen of Law  │   │ Royal Core    │
                    │ Merkle /      │   │ Capabilities  │   │ Truth + Law   │
                    │ Proof / Order │   │ Balance /     │   │ Order         │
                    └───────┬───────┘   └───────┬───────┘   └───────┬───────┘
                            │                   │                   │
                            └───────────────────┼───────────────────┘
                                                │
                                ┌───────────────▼───────────────┐
                                │          AMON / ISIS          │
                                │ Hidden Exec / Revival / RAM   │
                                └───────────────┬───────────────┘
                                                │
                                        ┌───────▼───────┐
                                        │    KHEPRI     │
                                        │ Init / Wake   │
                                        │ Boot / Birth  │
                                        └───────┬───────┘
                                                │
                                        ┌───────▼───────┐
                                        │      RE       │
                                        │ Supervisor    │
                                        │ CPU / Memory  │
                                        └───────┬───────┘
                                                │
                     ┌──────────────────────┼───────────────────────┐
                     │                      │                       │
                     ▼                      ▼                       ▼
          ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
          │      RA       │      │     PULSE     │      │      NIL      │
          │ app kernel    │      │ security k.   │      │ fs kernel     │
          │ plans / apps  │      │ crypto / guard│      │ filesystem    │
          └───────┬───────┘      └───────┬───────┘      └───────┬───────┘
                  │                          │                          │
                  │                          │                          │
                  ▼                          ▼                          ▼
       ┌───────────────┐        ┌─────────────────────────────┐   ┌───────────────────────┐
       │     PLAN      │        │   SHEKHMET / JI / ZUU      │   │     JINSHU / OCEAN    │
       │ manifest /    │        │   protection / integrity   │   │ storage / noun graph  │
       │ layers / cap  │        │   stealth / recovery       │   │ fs data layer         │
       └───────┬───────┘        └───────────────┬─────────────┘   └───────────┬───────────┘
               │                                │                               │
               ▼                                ▼                               ▼
       ┌───────────────┐               ┌───────────────┐               ┌───────────────┐
       │     PRISM     │               │     DOUAT      │               │     NOUN      │
       │ plan runtime  │               │ spectral net  │               │ content IDs   │
       │ sandbox       │               │ hidden paths  │               │ 32-byte refs  │
       └───────┬───────┘               └───────┬───────┘               └───────────────┘
               │                                │
               │                                ▼
               │                     ┌───────────────────────┐
               │                     │        AMMIT          │
               │                     │  death switch / kill │
               │                     │  final cutoff        │
               │                     └───────────────────────┘
               │
               ▼
       ┌───────────────┐
       │     ABYSS     │
       │ inverse of    │
       │ prism / plan  │
       │ unraveler     │
       └───────────────┘
```

![Amentys](amentys.png)

                                │ erase / burn               │
                                └───────────────┬─────────────┘
                                                │
                                                ▼
                                ┌─────────────────────────────┐
                                │          ANUBIS            │
                                │ hypervisor / watchers     │
                                │ boundary                  │
                                └─────────────────────────────┘
```

![Amentys](amentys.png)

## Devel requirement

* lib
  * blake3
* binary
  * rustup (nightly)
  * gcc
  * make
  * blake3
  * git
  * bootimage
  * qemu

## Documentation

In order to show documentation run `make doc`.

* [Filesystem](FILESYSTEM.md)
