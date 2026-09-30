# Ferentin mirror of brush

This is a mirror of [reubeno/brush](https://github.com/reubeno/brush) (MIT).
Ferentin depends on `brush-parser` from it through a pinned git revision in
[ferentin-endpoint](https://github.com/ferentin-net/ferentin-endpoint).

## Why this mirror exists

We use `brush-parser` to parse shell command lines that come from an untrusted
source: the commands AI agents run on a governed device. A malformed or hostile
command line has to come back as an error, promptly. It must not take the
process down or hold the thread that parses it.

## What is changed

`main` is upstream `8b539dd` plus four commits, all confined to `brush-parser`
except the CI change:

| Commit | What it does | Upstream |
|---|---|---|
| `fix(parser): stop nested case clauses taking exponential time` | Each level of a nested `case` parsed the level below it twice when an item had no `;;`, so about 24 levels (330 bytes) of valid shell took more than 10 seconds. | [#1420](https://github.com/reubeno/brush/pull/1420) |
| `fix(parser): bound the grammar's recursion depth` | Nesting between tokens had no bound, so 5,000 levels of brace groups, `if`, `case`, `[[ ! ]]` and more aborted on a stack overflow. Declines past 32 levels with `ParseError::NestingTooDeep`. | [#1420](https://github.com/reubeno/brush/pull/1420) |
| `Cap CI artifact retention at 7 days` | Keeps this mirror's CI artifacts from filling the org's Actions storage. Mirror only. | never |
| `Let a caller put a deadline on the word grammar` | `word::set_parse_deadline` and `word::parse_deadline_expired`. Bounds the word grammar's remaining superlinear shapes, such as `${x[a[a[...` and unmatched `{a,`, by time. Unarmed, the grammar is unchanged. | not yet offered |

Upstream has since fixed the rest of what this mirror used to carry:

- The tokenizer hang on an unterminated here tag, and unbounded `$(...)`
  nesting inside a token: our [#1270](https://github.com/reubeno/brush/pull/1270),
  folded into upstream [#1416](https://github.com/reubeno/brush/pull/1416).
- The two out-of-range numeric panics: our
  [#1271](https://github.com/reubeno/brush/pull/1271), merged.
- The exponential word grammar on unmatched openers inside `$(...)`, which #4
  and #5 here memoized: upstream
  [#1407](https://github.com/reubeno/brush/pull/1407) removed that path, and
  those commits are no longer carried.

## Keeping up with upstream

`main` is the durable ref that ferentin-endpoint pins. It is never rebased or
force-pushed, so every revision the endpoint has ever pinned stays fetchable.

To take a newer upstream, build the new patch set on a branch from upstream
`main`, then land it on `main` as a merge whose tree is that branch's tree:

```sh
git fetch upstream
git checkout -b ferentin/upstream-<sha> upstream/main
# cherry-pick or port each commit in the table above
cargo test -p brush-parser
cargo clippy --workspace --all-targets --all-features -- -D warnings

git checkout -b merge-<sha> origin/main
git merge --no-ff --no-commit -s ours ferentin/upstream-<sha>
git read-tree -u --reset ferentin/upstream-<sha>
git commit
```

Then open a pull request from `merge-<sha>` into `main`.

Look for resolutions a textual merge does not flag. Upstream `8889933` added an
entry point into the `token_parser` grammar, `compound_assignment_value`. It
merged cleanly and then failed to build, because this patch set gives that
grammar a `NestingTracker` parameter. Every new entry point needs a tracker of
its own.

When a commit here lands upstream, drop it on the next update. When upstream
rewrites the code a commit here changes, check whether the problem still
reproduces before porting the commit. The memoization commits were dropped that
way.

## Checking a change to the grammar

Snapshot tests did not catch the two regressions the first version of #1420 had.
A comparison did: parse a large set of inputs with the change and without it, and
compare the full `Debug` output of each result, or its error message, line by
line. Enumerate every short token sequence in the part of the grammar the change
touches, and add every script in `brush-shell/tests/cases`. The #1420 description
lists the sets used.

## Related, not ours

Upstream [#948](https://github.com/reubeno/brush/issues/948) looks related and is
not. It reports the same symptom, a stack overflow abort, but the cause is
runtime recursion through a self-referencing shell function
(`nproc(){ nproc; }` then `echo $(nproc)`), not parse-time nesting.

## License

Upstream is MIT and remains so. `LICENSE` is unmodified. This mirror carries no
additional restriction on the upstream code.
