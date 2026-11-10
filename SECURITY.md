# Security Policy

pamine parses untrusted network input and runs third-party Lua scripts, so
security reports are taken seriously.

## Supported versions

The project is pre-1.0 and moves fast. Only the latest commit on `main` is
supported — there are no backports to older commits or tags.

| Version | Supported |
|---|---|
| `main` (latest) | ✅ |
| anything older | ❌ |

## Reporting a vulnerability

**Please do not open a public issue for a security problem.**

Report it privately to:

**rarekiller777@gmail.com**

Include as much of the following as you can:

- what the issue is and which component it affects (`pcore`, `pcore-gen`, `app`,
  the Lua engine, the parser, …);
- how to reproduce it — a minimal packet capture, a Lua snippet, or a test case
  is ideal;
- the impact you believe it has (crash, memory exhaustion, code execution,
  sandbox escape, …);
- the commit you tested against.

If you want to encrypt the report, say so in a first plain-text message and a
key will be provided.

## What to expect

- **Acknowledgement** within a few days.
- An assessment of the report and, if it is valid, an estimate of when a fix
  will land.
- Credit in the fix commit and release notes, unless you ask to stay anonymous.

Please give a reasonable amount of time for a fix before disclosing publicly.

## Scope

In scope:

- **Parser and decoder** — malformed or hostile packets that cause a panic,
  unbounded allocation, or desynchronisation of the stream. The decoder is
  deliberately incomplete, so "this packet is not modelled yet" is not a
  vulnerability; a crash or a memory blow-up is.
- **Framing and compression** — length-prefix handling, the `Set Compression`
  header, and zlib payloads, including decompression bombs.
- **Lua sandbox** — escaping the restricted standard library, reaching Rust
  memory, or making the engine execute arbitrary code outside the script.
- **Script loading** — path traversal in a `.tar` archive, or a manifest that
  makes the loader read outside the script root.
- **Privilege handling** — anything that lets the process keep or misuse
  `cap_net_admin` / `cap_net_raw`.

Out of scope:

- Vulnerabilities in Minecraft itself or in the vanilla server/client.
- The fact that a script can read and modify the traffic of the connection it
  is attached to — that is the entire point of the project.
- Denial of service that requires the attacker to already control the local
  machine or the script being run.
- Missing packet definitions. Report those as normal issues.

## Handling of scripts

Scripts are **not** sandboxed against the connection they run on: a script can
read every packet and inject anything. Only run scripts you trust. The planned
script hub (see the roadmap in [README.md](README.md)) will add manual
moderation, but moderation is not a security boundary — treat downloaded
scripts with the same caution as any other executable content.
