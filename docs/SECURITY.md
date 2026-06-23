# Security Policy

CompressO CLI takes security seriously. This document describes what is in
scope, the security model the tool implements, and how to report a
vulnerability.

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security problems.**

Please report suspected vulnerabilities privately:

1. Go to the repository's **Security** tab → **Advisories** → "Report a
   vulnerability", **or**
2. Email the maintainer directly (see the project profile for the contact).

When reporting, please include:

- CompressO CLI version (`compresso --version`).
- FFmpeg version (`ffmpeg -version`) and how it was resolved
  (`COMPRESSO_FFMPEG_PATH`, bundled, or `PATH`).
- Operating system and version.
- A minimal reproduction: the exact command, the input that triggers it, and
  the observed vs. expected behavior.
- Whether the issue is exploitable on a default install, or only under a
  non-default configuration.

You should receive an initial response within **5 business days**. Please
refrain from public disclosure until a fix (or coordinated disclosure date) has
been agreed.

## Scope

### In scope

- Path traversal / symlink-based write attacks against **output paths**
  (writing files outside the intended directory, into system directories, or
  clobbering protected files).
- Command injection or argument injection via filenames or options.
- FFmpeg binary resolution weaknesses (e.g. PATH hijacking leading to
  execution of a malicious `ffmpeg`).
- TOCTOU races in the temp-file / atomic-rename pipeline.
- Loss of cleanup (temp files left behind in a way that leaks sensitive data).
- Crashes / panics triggered by malformed input that could be used for
  denial-of-service in an automated pipeline.

### Out of scope

- Vulnerabilities in **FFmpeg itself** — report those upstream to the FFmpeg
  project. CompressO CLI is a thin wrapper and does not bundle FFmpeg.
- Issues that require an attacker to already control the input video *and*
  the FFmpeg binary resolution (full compromise is already assumed in that
  model).
- Social engineering, physical access, or compromising the user's account.

## Security model

CompressO CLI implements defense-in-depth for filesystem operations:

| Layer | What it does |
|-------|--------------|
| **Path canonicalization** | Input and output paths are canonicalized so symlinks resolve to their real target before any check or write. |
| **System-directory denylist** | The *canonicalized* output path is matched against a list of protected locations (`/etc`, `/usr`, `/sys`, `/proc`, `/dev`, `/boot`, `/root`, `/lib`, `/bin`, `/sbin`, `/var`, `/run`, macOS `/System`, `/Library`, `/Applications`, Windows `C:\Windows`, `C:\Program Files`, `C:\ProgramData`, etc.). Writes to a filesystem / drive root are refused. |
| **Traversal rejection** | `..` sequences and null bytes in paths are rejected outright (not merely warned about). |
| **Filename sanitization** | Auto-generated output names strip path separators, control characters, null bytes, and `..`, while preserving legitimate non-ASCII characters (Cyrillic, CJK, …). |
| **Atomic writes** | Output is written to a uniquely-named temp file in the target directory and atomically renamed into place on success. The temp file is removed on every exit path (including cancellation) via an RAII guard. |
| **FFmpeg resolution priority** | `COMPRESSO_FFMPEG_PATH` (explicit, most secure) → bundled FFmpeg (optionally verified) → system `PATH` (least secure, logs a warning). |

### Verbatim-prefix handling (Windows)

`std::fs::canonicalize` on Windows returns paths with the `\\?\` (or
`\\?\UNC\`) prefix. CompressO CLI strips this prefix before matching against
the denylist, so the canonical form and the denylist use a consistent
representation. See `strip_verbatim_prefix` in `src/ffmpeg.rs`.

## Hardening recommendations (deployment)

- Set `COMPRESSO_FFMPEG_PATH` to an absolute, trusted FFmpeg binary in
  production / CI environments.
- Set `COMPRESSO_FFMPEG_VERIFY=1` if shipping a bundled FFmpeg, to require
  that the bundled binary passes a basic `--version` sanity check.
- Run the tool as a non-privileged user and in a working directory dedicated
  to the input/output set.
- Review stderr warnings — security-relevant events (PATH-based resolution,
  rejected paths) are logged there.
