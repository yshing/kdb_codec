# Fuzzing Guide for kdb_codec

This document explains how to run fuzz tests on the kdb_codec library to discover security vulnerabilities.

## Prerequisites

### 1. Install Nightly Toolchain

```bash
rustup toolchain install nightly
```

### 2. Add Required Target (macOS ARM/Apple Silicon)

If you're on Apple Silicon (M1/M2/M3):
```bash
rustup target add aarch64-apple-darwin --toolchain nightly
```

If you're on Intel Mac:
```bash
rustup target add x86_64-apple-darwin --toolchain nightly
```

If you're on Linux:
```bash
rustup target add x86_64-unknown-linux-gnu --toolchain nightly
```

### 3. Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

Note: cargo-fuzz requires a nightly Rust toolchain with the appropriate target installed.

## Important: Working Directory

**All fuzzing commands must be run from the `kdb_codec` directory:**

```bash
cd kdb_codec
```

All examples below assume you are in the `kdb_codec` directory.

## Running Fuzz Tests

### 1. Fuzz `q_ipc_decode` (Deserialization)

This tests arbitrary byte sequences against the deserialization logic:

```bash
# From kdb_codec directory
cargo +nightly fuzz run fuzz_q_ipc_decode
```

**What it finds:**
- Panics from invalid type bytes
- Buffer overruns in list deserialization
- Integer overflows in size calculations
- UTF-8 validation issues
- Unbounded recursion

### 2. Fuzz `decompress_sync` (Decompression)

This specifically targets the decompression algorithm:

```bash
# From kdb_codec directory
cargo +nightly fuzz run fuzz_decompress
```

**What it finds:**
- Out-of-bounds reads in decompression loop
- Invalid back-references
- Decompression bombs
- Size field validation issues

### 3. Fuzz Complete Codec Decode Path

This tests the entire codec decoding pipeline:

```bash
# From kdb_codec directory
cargo +nightly fuzz run fuzz_codec_decode
```

**What it finds:**
- Header validation bypasses
- Message size handling issues
- Integration bugs between components

## Running with Memory Limit

To prevent the fuzzer from being killed by OOM, set a memory limit:

```bash
# Limit to 2GB RSS
cargo +nightly fuzz run fuzz_q_ipc_decode -- -rss_limit_mb=2048
```

## Running with Time Limit

```bash
# Run for 1 hour
cargo +nightly fuzz run fuzz_q_ipc_decode -- -max_total_time=3600
```

## Checking for Specific Issues

### Test for Slow Inputs (Hangs)

```bash
# Timeout after 10 seconds per input
cargo +nightly fuzz run fuzz_q_ipc_decode -- -timeout=10
```

### Test with Dictionary for Structure-Aware Fuzzing

The fuzzer can learn from valid inputs to generate more interesting test cases:

```bash
# Add valid kdb+ messages to the corpus
mkdir -p fuzz/corpus/fuzz_q_ipc_decode
# Add some valid serialized K objects here
cargo +nightly fuzz run fuzz_q_ipc_decode
```

## Analyzing Crashes

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/`:

```bash
# Reproduce a crash
cargo +nightly fuzz run fuzz_q_ipc_decode fuzz/artifacts/fuzz_q_ipc_decode/crash-xyz
```

## Minimizing Crash Cases

Reduce a crashing input to its minimal form:

```bash
cargo +nightly fuzz cmin fuzz_q_ipc_decode
```

## Coverage Report

Generate a coverage report to see what code paths are being tested:

```bash
cargo +nightly fuzz coverage fuzz_q_ipc_decode
```

## Continuous Fuzzing

For continuous integration, run fuzzing for a fixed time:

```bash
#!/bin/bash
# fuzz.sh - Run all fuzzers for 10 minutes each

FUZZ_TIME=600  # 10 minutes

echo "Fuzzing q_ipc_decode..."
cargo +nightly fuzz run fuzz_q_ipc_decode -- -max_total_time=$FUZZ_TIME

echo "Fuzzing decompress..."
cargo +nightly fuzz run fuzz_decompress -- -max_total_time=$FUZZ_TIME

echo "Fuzzing codec decode..."
cargo +nightly fuzz run fuzz_codec_decode -- -max_total_time=$FUZZ_TIME

echo "Fuzzing complete!"
```

## Expected Issues (Before Fixes)

Based on the security review, fuzzing is likely to find:

1. **Panics in `decompress_sync`**
   - Invalid compressed data
   - Size field < 8
   - Out-of-bounds reads

2. **Panics in deserialization**
   - Missing null terminators in symbols
   - Invalid UTF-8 sequences
   - Stack overflow from deep nesting

3. **Memory exhaustion**
   - Large size fields causing huge allocations
   - Decompression bombs

4. **Out-of-bounds access**
   - Invalid back-references in decompression
   - Buffer overruns in list construction

## After Implementing Fixes

After fixing the security issues:
- Panics should be eliminated (return errors instead)
- Memory should be bounded (size limits enforced)
- All bounds should be checked
- Run fuzzing for longer periods (hours/days) in CI

## Integration with CI

Add to `.github/workflows/fuzzing.yml`:

```yaml
name: Fuzzing

on:
  schedule:
    - cron: '0 2 * * *'  # Run nightly
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzzing
        run: |
          cd kdb_codec
          cargo +nightly fuzz run fuzz_q_ipc_decode -- -max_total_time=3600
      - name: Upload artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-artifacts
          path: kdb_codec/fuzz/artifacts/
```

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Trophy Case](https://github.com/rust-fuzz/trophy-case)
