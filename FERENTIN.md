# Ferentin mirror of brush

This is a mirror of [reubeno/brush](https://github.com/reubeno/brush) (MIT).
Upstream is unmodified on `main`. Our changes live on `ferentin-patches`.

## Why this mirror exists

We use `brush-parser` to parse shell command lines that come from an untrusted
source, so a malformed command line has to return an error rather than take the
process down with it. A spike against `brush-parser` 0.4.0 found four ways a
short command line could do exactly that, and following the stack overflow up
found a fifth family the first fix did not cover.

None of them are exotic. The worst is seven bytes.

## What is changed

Three commits on `ferentin-patches`, all confined to `brush-parser`:

| Commit | Fixes |
|---|---|
| `fix(parser): stop the tokenizer hanging and overflowing on hostile input` | An unterminated here tag containing an unterminated expansion spun forever at end of input, allocating each pass, reaching roughly 10 GB resident in seconds. Nested `$(...)` recursion was unbounded and aborted on stack overflow. |
| `fix(parser): decline out-of-range numeric values instead of unwrapping` | Two grammar rules unwrapped an integer parse, so `echo 99999999999999999999>&1` and `~99999999999999999999` panicked the parser. |
| `fix(parser): bound the grammar's recursion depth` | The tokenizer's bound covers nesting *inside* a token; nesting *between* tokens is a separate descent that had no bound. Nested brace groups, `if`, `while`, `coproc`, subshells, process substitutions, `[[ ]]` parentheses and negations, arithmetic parentheses and regex groups each aborted on stack overflow -- the cheapest of them at 120 levels. |

Each commit builds and tests on its own, so the history stays bisectable and
either commit can be cherry-picked out for an upstream pull request.

## Verification

Against the upstream test suites, on `aarch64-apple-darwin`:

- `cargo test -p brush-parser`: 236 passed, 0 failed. That is upstream's 224 plus
  12 regression tests added here.
- `cargo clippy -p brush-parser --all-targets`: clean, under upstream's own
  `all` / `pedantic` / `nursery` / `cargo` configuration with warnings denied.
- `cargo test -p brush-shell --test brush-compat-tests`: 1389 succeeded, 404
  failed, 366 known to fail, 43 skipped. Compared case by case against a
  pristine checkout, the only differences are process-substitution cases that
  come out differently on consecutive runs of the *same* build, because the
  bash 3.2 oracle races on them. The failures are pre-existing and mostly
  reflect that local bash.
- Two 500,000 iteration structured fuzz runs over shell metacharacters: no
  panics. Before these fixes the same fuzzer found a panic within the first
  200,000 iterations and an out of memory kill at iteration 155,392.

Behaviour for each fixed input was checked against real bash rather than chosen
for convenience. Where we knowingly diverge, the commit message says so.

## Keeping up with upstream

`main` tracks upstream and should never carry our changes.

```sh
git fetch upstream
git checkout main
git merge --ff-only upstream/main
git push origin main

git checkout ferentin-patches
git rebase main
cargo test -p brush-parser && cargo clippy -p brush-parser --all-targets
git push --force-with-lease origin ferentin-patches
```

If a rebase drops one of our commits cleanly, that means upstream fixed the same
bug, which is the outcome we want.

## Upstreaming

These are upstream bugs and belong upstream. Each commit is self-contained and
was written to be cherry-picked onto a fork of `reubeno/brush` and sent as a
pull request, which is the intended end state for this mirror: once the fixes
land upstream and a release carries them, a rebase drops our commits and the
mirror stops being needed.

None of the four had an issue filed at the time of the spike. The grammar
recursion bound is tracked here as
[#1](https://github.com/ferentin-net/brush/issues/1).

Upstream [#948](https://github.com/reubeno/brush/issues/948) looks related and is
not. It reports the same symptom, a stack overflow abort, but the cause is
runtime recursion through a self-referencing shell function
(`nproc(){ nproc; }` then `echo $(nproc)`), not parse-time nesting. It still
reproduces with our fixes applied, so we do not claim it.

## License

Upstream is MIT and remains so. `LICENSE` is unmodified. This mirror carries no
additional restriction on the upstream code.
