# Amentys

[![Amentys](https://github.com/kognitara/amentys/actions/workflows/amentys.yml/badge.svg)](https://github.com/kognitara/amentys/actions/workflows/amentys.yml)

Amentys is a layered operating and runtime architecture built around a small, explicit trust model.
The system is organized around a supervisor, application kernels, filesystem and storage layers, and protection/security primitives.

```txt
                   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
                   │     THOT      │   │     MAAT      │   │   AMENTYS     │
                   │ truth / proof │   │ law / balance │   │ core / order  │
                   │ order / root  │   │ capability    │   │ authority     │
                   └───────┬───────┘   └───────┬───────┘   └───────┬───────┘
                           │                   │                   │
                           └───────────────────┼───────────────────┘
                                               │
                               ┌───────────────▼───────────────┐
                               │        AMON / ISIS            │
                               │ hidden exec / revival / ram   │
                               └───────────────┬───────────────┘
                                               │
                                       ┌───────▼───────────┐
                                       │       KHEPRI      │
                                       │ init / wake / boot│
                                       └───────┬───────────┘
                                               │
                                       ┌───────▼───────────────┐
                                       │          RE           │
                                       │ supervisor / cpu /    │
                                       │ memory / runtime      │
                                       └───────┬───────────────┘
                                               │
                 ┌─────────────────────────────┼─────────────────────────────┐
                 │                             │                             │
                 ▼                             ▼                             ▼
      ┌───────────────┐           ┌───────────────┐           ┌───────────────┐
      │      RA       │           │     PULSE     │           │      NIL      │
      │ app kernel    │           │ security      │           │ fs kernel     │
      │ plans / apps  │           │ guard / crypto│           │ filesystem    │
      └───────┬───────┘           └───────┬───────┘           └───────┬───────┘
              │                           │                           │
              ▼                           ▼                           ▼
      ┌───────────────┐           ┌─────────────────────────┐    ┌────────────────┐
      │     PLAN      │           │   SHEKHMET / JI / ZUU   │    │ JINSHU / OCEAN │
      │ manifest /    │           │ protection / integrity  │    │ storage / data │
      │ capabilities  │           │ stealth / recovery      │    │ graph / noun   │
      └───────┬───────┘           └───────────┬─────────────┘    └───────┬────────┘
              │                               │                          │
              ▼                               ▼                          ▼
      ┌───────────────┐               ┌───────────────┐               ┌───────────────┐
      │     PRISM     │               │     DOUAT     │               │     NOUN      │
      │ runtime /     │               │ spectral paths│               │ ids / refs    │
      │ sandbox       │               │ hidden routes │               │ 32-byte refs  │
      └───────┬───────┘               └───────┬───────┘               └───────────────┘
              │                               │
              │                               └───────────────┬───────────────┐
              │                                               │               │
              ▼                                               ▼               ▼
      ┌───────────────┐                                 ┌───────────────┐ ┌───────────────┐
      │     ABYSS     │                                 │     AMMIT     │ │    ANUBIS     │
      │ inverse /     │                                 │ cutoff / kill │ │ watcher /     │
      │ unraveler     │                                 │ final border  │ │ boundary      │
      └───────────────┘                                 └───────────────┘ └───────────────┘
```

![Amentys](amentys.png)

## Development requirements

### Libraries

- blake3

### Binaries and tooling

- rustup (nightly)
- gcc
- make
- blake3
- git
- bootimage
- qemu

## Documentation

To build the project documentation:

```bash
make doc
```

- [Filesystem](FILESYSTEM.md)
- [License](LICENSE)
- [Third-party notices](NOTICE)
- [Security policy](SECURITY.md)
- [Privacy and telemetry](PRIVACY.md)
- [Trademarks and branding](TRADEMARKS.md)
- [Contribution rules](CONTRIBUTING.md)
