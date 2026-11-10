# Contributing to pamine

Thanks for wanting to help. This document covers the practical side: how to
build, how to test, and what a good change looks like.

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Before you start

- **Open an issue first** for anything larger than a bug fix or a small
  cleanup. It is much cheaper to agree on an approach than to redo a pull
  request.
- Check the [roadmap](README.md#roadmap) — the thing you want to work on may
  already be planned, or deliberately deferred.
- Security problems go to [SECURITY.md](SECURITY.md), **not** to a public issue.

---

## Setting up

Requirements:

- a recent stable Rust toolchain (the crates use `edition = "2024"`);
- `tar` and a POSIX shell, for the test-script build script;
- on Linux, `sudo` for `setcap` if you want to run the relay for real.

```bash
git clone git@github.com:Sonys9/pamine.git
cd pamine

cd app && cargo build
```

The workspace is three independent crates, not a Cargo workspace, so build and
test them separately:

```bash
cd pcore     && cargo test
cd pcore-gen && cargo test
cd app       && cargo test
```

`pcore-gen` uses [`trybuild`](https://docs.rs/trybuild), so its test suite
compiles the files under `pcore-gen/tests/compile-fail/` and compares the
compiler output against the `.stderr` snapshots. If you change a derive's error
message, those snapshots need updating — run with `TRYBUILD=overwrite` and
**read the diff** before committing it.

---

## Running it

```bash
cd app
cp .env.template .env     # set BIND_ADDR and MARK
./run.sh
```

`run.sh` builds in release mode, grants `cap_net_admin` and `cap_net_raw` to the
binary, and runs it. The capabilities are needed because the relay reads the
original destination of a redirected connection.

---

## The test script

The Lua test script lives in `app/tests/scripts/`:

```
app/tests/scripts/
├── build.sh              # rebuilds the archive
├── test_script.tar       # built artifact, committed
└── test_script/          # sources
```

The archive is committed because the tests `include_bytes!` it, so it must exist
at compile time. **After editing anything under `test_script/`, rebuild it:**

```bash
./app/tests/scripts/build.sh
```

Forgetting this is the most common way to get a confusing test failure: the
tests read the archive, not the sources.

---

## Code style

- `cargo fmt` before committing. The repo is formatted with the default
  rustfmt settings.
- `cargo clippy` should not add new warnings. Existing warnings are known and
  listed in the roadmap; do not silence a new one with `#[allow]` without a
  comment explaining why.
- Comments explain **why**, not what. A comment that restates the code is
  noise; a comment that records a non-obvious constraint (a wire-format quirk, a
  lifetime that cannot be avoided, a bug that was fixed) is valuable.
- Prefer a small, focused function over a long one with section comments.
- Error messages are read by users. Say what was expected and what was found.

### Naming

- Packet structs are named after the protocol packet (`SetEntityMetadata`), and
  the enum variant matches the struct name.
- Lua-facing names mirror the Rust nesting: `s2c.play.SetEntityMetadata`.

---

## Adding a packet

Packet definitions are declarative — the derives generate the wire format, the
Lua table conversion and the Lua constructor from one declaration.

1. Add the struct to the right submodule under
   `app/src/packets/{c2s,s2c}/play/` (or the phase module for
   handshake/login/status).
2. Derive the full set:

   ```rust
   #[derive(Decode, Encode, Lua, LuaPacket, Debug, Clone)]
   pub struct MyPacket<'a> {
       pub id: VarInt,
       pub name: &'a str,
   }
   ```

3. Add the variant to the phase enum with the correct opcode:

   ```rust
   #[id(0x42)]
   MyPacket(MyPacket<'a>),
   ```

4. Add a round-trip test. For a packet with a captured body, put the bytes in
   `app/tests/data/` and `include_bytes!` them, like
   [`command.rs`](app/src/packets/s2c/play/command.rs) does.

Field types map to the wire format automatically; the table in
[`pcore-gen/src/lua_packet.rs`](pcore-gen/src/lua_packet.rs) documents the
mapping. If a field needs special treatment, use the existing attributes
(`#[decode(rest)]`, `#[decode(len = "...")]`, `#[lua(skip)]`,
`#[lua(validate = "...")]`) rather than hand-writing an impl.

---

## Adding a Lua binding

- Bindings for protocol primitives live in `pcore/src/lua.rs`, because the
  orphan rule forbids implementing `IntoLua`/`FromLua` for foreign types in
  `app`.
- Bindings for app types live in `app/src/scripts/adaptations/`.
- A type that borrows from the packet buffer (`&'a str`, `&'a [u8]`,
  `NbtTag<'a>`, `Slot<'a>`) cannot implement `FromLua` without leaking. Provide
  an owned mirror instead, the way `OwnedNbtTag` and `OwnedSlot` do.

---

## Commit messages

Conventional Commits, with a scope naming the area:

```
feat(scripts): register handlers by packet constructor
fix(commands): minecraft:time has no properties in 1.19.2
ref(scripts/tests): move test script fixtures out of src
docs: add roadmap and contribution guides
test(parser): cover a decompression bomb
```

Types in use: `feat`, `fix`, `ref`, `docs`, `test`, `chore`.

The body should explain **why** the change is needed. If the change fixes a bug,
describe the failure mode — that is what makes the commit useful a year later.

Keep commits focused. A refactor and a behaviour change in one commit are hard
to review and hard to revert.

---

## Pull requests

- One logical change per pull request.
- `cargo test` passes in every crate you touched.
- New behaviour comes with a test. Bug fixes come with a test that fails before
  the fix.
- If you change user-visible behaviour (the Lua API, the config, the CLI),
  update the README in the same pull request.
- Describe what you changed and why. If there is a trade-off, say which way you
  went and what the alternative was.

---

## Questions

Open an issue, or write to **rarekiller777@gmail.com**.
