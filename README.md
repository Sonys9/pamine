# pamine

**Minecraft Packet-Level Game Enhancer**

pamine sits between a Minecraft client and a server, parses every packet on the
wire, and lets Lua scripts inspect, rewrite and inject traffic. It is not a
proxy in the "forward bytes blindly" sense: packets are decoded into typed Rust
structs, handed to scripts as Lua tables, and re-encoded on the way out.

Target protocol: **760 (Minecraft 1.19.2)**.

---

## Table of contents

- [How it works](#how-it-works)
- [Repository layout](#repository-layout)
- [Quick start](#quick-start)
- [Writing a script](#writing-a-script)
- [Development](#development)
- [Roadmap](#roadmap)
- [License](#license)

---

## How it works

```
client ──▶ socket ──▶ parser ──▶ Packet<'a> ──▶ Lua handler ──▶ Update
                                                                    │
                          ┌─────────────────────────────────────────┘
                          ▼
                    rewrite / inject ──▶ frame ──▶ socket ──▶ server
```

1. **Framing** — [`app/src/packets/parser/frame.rs`](app/src/packets/parser/frame.rs)
   splits the TCP stream into length-prefixed frames and handles the
   `Set Compression` header and zlib payload.
2. **Dispatch** — [`app/src/packets/parser/dispatch.rs`](app/src/packets/parser/dispatch.rs)
   reads the opcode and decodes the body into the right enum variant for the
   current protocol phase (`Handshake` → `Login` → `Play` / `Status`).
3. **Scripting** — [`app/src/scripts/engine.rs`](app/src/scripts/engine.rs)
   looks up the handler registered for `(side, state, opcode)` and calls it with
   `{ packet, threshold, is_injected }`.
4. **Rewrite** — the handler returns an `Update` describing whether to replace
   the packet, inject extra packets, both, or nothing.

A packet body that fails to decode is **not** fatal: the frame boundary is
already known, so the frame is forwarded untouched and the connection stays in
sync. This matters because the decoder is not complete yet.

### Crates

| Crate | Role |
|---|---|
| [`pcore`](pcore) | Protocol primitives: `Decoder`, `Encoder`, `VarInt`, `NbtTag`, `Slot`, `Uuid`, framing, and the Lua bindings for them. |
| [`pcore-gen`](pcore-gen) | Derive macros: `Decode`, `Encode`, `DecodePacket`, `EncodePacket`, `Lua`, `LuaPacket`. |
| [`app`](app) | The binary: sockets, connection state machine, packet definitions, Lua engine. |

### Packet definitions

Every packet is a plain Rust struct, and each protocol phase is an enum whose
variants carry `#[id(N)]`:

```rust
#[derive(DecodePacket, EncodePacket, Lua, LuaPacket, Clone, Debug)]
pub enum Play<'a> {
    #[id(0x50)]
    SetEntityMetadata(SetEntityMetadata<'a>),
    // ...
}
```

The derives generate, from that one declaration:

- `decode_by_opcode` / `encode` — the wire format;
- `IntoLua` — the table a handler receives;
- `lua_bindings` — the `s2c.play.SetEntityMetadata` constructor a script calls.

---

## Repository layout

```
.
├── app/
│   ├── build.rs              # generates block/registry tables from data/*.json
│   ├── data/                 # blocks.json, registries.json (build input)
│   ├── src/
│   │   ├── connections/      # sockets, relay, rewrite, injection
│   │   ├── packets/          # c2s/ and s2c/ definitions, parser, dispatch
│   │   ├── scripts/          # Lua engine, loader, adaptations
│   │   └── main.rs
│   └── tests/
│       ├── data/             # captured packet bodies used by tests
│       └── scripts/          # test script sources + built test_script.tar
├── pcore/                    # protocol primitives
└── pcore-gen/                # derive macros
```

---

## Quick start

Requirements: a recent stable Rust toolchain (the crates use `edition = "2024"`).

```bash
cd app
cp .env.template .env        # then edit BIND_ADDR and MARK
./run.sh                     # builds release, sets cap_net_admin/cap_net_raw, runs
```

`run.sh` needs `sudo` for `setcap`, because the relay uses `SO_ORIGINAL_DST` to
find the real destination of a redirected connection.

> **Status:** the binary currently stops right after the scripting smoke test
> (`todo!("Stopped after doing the scripting stuff")` in
> [`app/src/main.rs`](app/src/main.rs)). Wiring the engine into the live
> connection loop is roadmap item 1.

---

## Writing a script

A script is a `.tar` archive with a `Script.toml` manifest and Lua sources:

```toml
[script]
name = "My script"
engine_version = 1
description = "What it does"
uuid = "dafe77bb-e1d5-4560-b473-288676192212"
main_file = "main.lua"
```

Handlers are registered by **packet constructor**, not by a numeric opcode:

```lua
engine:register("Play", "S2C", s2c.play.SetEntityMetadata, "read-inject", function(opts)
    print("entity", opts.packet.value.entity_id)

    local packet = s2c.play.SetEntityMetadata {
        entity_id = 1,
        metadata = {
            { index = 0, value = { type = "Particle", particle_id = 24,
              data = { type = "Dust", red = 1.0, green = 0.0, blue = 0.0, scale = 1.0 } } },
        },
    }

    local inject = update.injection(true)
    inject:push_packet(packet, opts.threshold)
    return update.inject(inject)
end)
```

The constructor carries its own `opcode`, `side` and `state`, so the engine can
reject a registration that names the wrong phase instead of silently never
firing.

**Access levels** (`own_type`):

| Value | May read | May replace | May inject |
|---|:---:|:---:|:---:|
| `"read"` | ✅ | ❌ | ❌ |
| `"read-write"` | ✅ | ✅ | ❌ |
| `"read-inject"` | ✅ | ❌ | ✅ |
| `"read-write-inject"` | ✅ | ✅ | ✅ |

The engine enforces this at runtime: a `"read"` handler that returns an `Update`
is an error, not a silent no-op.

---

## Development

```bash
cd app       && cargo test     # 73 tests
cd pcore     && cargo test     # 54 tests
cd pcore-gen && cargo test     # 14 tests + trybuild compile-fail cases
```

Rebuild the bundled test script after editing its sources:

```bash
./app/tests/scripts/build.sh
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow.

---

## Roadmap

Planned work, broken into microtasks. Nothing here is implemented unless it is
checked off. Order is roughly the intended order, not a hard dependency chain.

### 1. Wire the Lua engine into the live connection loop

Right now the engine is only exercised by a smoke test in `main.rs`; the relay
does not call it.

- [ ] Replace the `todo!()` in [`app/src/main.rs`](app/src/main.rs) with the real accept loop.
- [ ] Give each connection its own `Engine` (or a shared one, decided by design) and load the script at connection start.
- [ ] Call `Engine::on_packet` from [`app/src/connections/rewrite.rs`](app/src/connections/rewrite.rs) for every parsed packet.
- [ ] Apply the returned `Update`: `change_with` replaces the frame, `inject` appends frames.
- [ ] Feed injected frames back through the parser so scripts can see each other's packets (the `is_injected` flag already exists for this).
- [ ] Decide and document what happens when a handler errors: drop the packet, drop the connection, or log and continue.
- [ ] Add an integration test that runs a script against a recorded packet stream.

### 2. `is_blocking` flag on registration

Handlers that `sleep` or otherwise await should not stall the packet pipeline.

- [x] Add `is_blocking: bool` to the registration table (see item 3).
- [ ] `is_blocking = true` — the handler is synchronous; run it inline, no task spawn.
- [ ] `is_blocking = false` — the handler may await; run it on a task and define the ordering guarantees for its `Update`.
- [ ] Add tests for both paths, including a handler that sleeps.

### 3. Pass registration info as a table

`engine:register("Play", "S2C", s2c.play.X, "read", fn)` has too many positional
arguments and is easy to get wrong.

- [ ] Accept a single table: `engine:register { packet = s2c.play.X, own_type = "read", is_blocking = true, handler = fn }`.
- [ ] Keep the positional form working for one release, with a deprecation warning.
- [ ] Validate the table: unknown keys are an error, missing required keys name the missing key.
- [ ] Derive `side` and `state` from the packet constructor instead of asking for them (already possible — the constructor carries both).
- [ ] Update the test script and the docs.

### 4. Internal packet parsing for selected packets

Some packets (chunks especially) are worth understanding, not just forwarding.

- [ ] Audit which packets actually matter for the enhancer's use cases and write the list down.
- [ ] Implement chunk section decoding (palette, block states, biomes, block entities).
- [ ] Expose decoded chunk data to Lua in a form that is cheap to read and does not copy the whole section.
- [ ] Decide the fallback for packets left undecoded: forward raw, as today.
- [ ] Add round-trip tests against captured chunk packets.

### 5. Optimised chunk and entity storage

- [ ] Design the storage layout: what is keyed by what, and what the lifetime of a chunk is.
- [ ] Add `zstd` compression, with a trained dictionary for chunk payloads.
- [ ] Apply the same approach to entity data.
- [ ] Measure: compression ratio and CPU cost on a realistic session.
- [ ] Add a cache eviction policy (chunks unload, entities despawn).
- [ ] Expose the store to Lua read-only first, then decide about writes.

### 6. Web panel

- [ ] Pick the stack: `axum` for the backend, `htmx` for the frontend (light on both bytes and effort).
- [ ] Serve a read-only dashboard: active connections, packets per second, script status.
- [ ] Add script management: upload, enable/disable, view logs.
- [ ] Add authentication — the panel must not be open by default.
- [ ] Bind to localhost by default and document how to expose it safely.

### 7. Script hub server

- [ ] Stand up a small `axum` service for publishing and downloading scripts.
- [ ] Manual moderation queue: submitted scripts are pending until approved or rejected.
- [ ] Store the moderation decision and the reviewer with each script.
- [ ] Add a client-side command to install a script from the hub by id.
- [ ] Sign or hash published archives so a client can verify what it downloaded.

### 8. Code cleanup and test coverage

- [ ] Re-read every module and note the rough edges.
- [ ] Refactor the connection layer: `testing_will_be_removed.rs` is a placeholder that should become a real module.
- [ ] Remove the dead-code warnings (`Parser::reset`, `StreamData::pending`, unused constants, …) or wire them up.
- [ ] Fill in the missing packet definitions and mark the ones that are known-incomplete.
- [ ] Raise test coverage on the parser and the Lua bridge, especially error paths.
- [ ] Add a fuzz target for the frame splitter and the decoder.

### 9. To be extended

This list grows as the project does.

- [ ] _(add the next item here)_

---

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).
