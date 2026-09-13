#!/usr/bin/env python3
"""Movement 5, part 4: remove the code the gate removal orphaned.

`cargo check` flagged four dead items, and all four existed only to serve the
CPAchecker gate:

  VerifyCtx::deadline / remaining_secs   its own doc says it exists so a stage that
                                         "shells out to a long-running external tool"
                                         can size its timeout. Nothing shells out now;
                                         every other stage carries its own `Instant`.
  CPACHECKER_TIMELIMIT_MARGIN_SECS       gate-only constant.
  overflow_source_constant_folds         the `-Winteger-overflow` probe inside
                                         `try_overflow_true`.

That last one also falsifies a doc line written in part 3, which claimed the probe
"fires independently". It does not -- it is gone. Fixed here.

Leaving the dead `deadline` in place would also break `cargo doc`: two doc comments
link to `[`cpachecker_timelimit`]`, which no longer exists.
"""
import re
import sys
from pathlib import Path

CLI = Path("crates/saf-cli/src/commands.rs")
SMOKE = Path("crates/saf-cli/tests/smoke.rs")

EXACT = [
    # --- VerifyCtx: drop the field and its accessor -------------------------
    ('''    /// When the caller's wall-clock watchdog will give up and emit `unknown`. Stages
    /// that shell out to a long-running external tool consult this so they finish
    /// INSIDE the budget instead of being `SIGKILLed` with the task (see
    /// [`cpachecker_timelimit`]). Stages with their own deterministic cost bound
    /// ignore it.
    deadline: std::time::Instant,
}

impl VerifyCtx<'_> {
    /// Whole seconds left before the watchdog fires (0 once it has passed).
    fn remaining_secs(&self) -> u64 {
        self.deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_secs()
    }
}
''', '''}
'''),

    # --- the watchdog no longer needs to hand a deadline to the strategy ----
    ('''/// Run compile -> ingest -> strategy on a worker thread under a wall-clock `budget`.
/// On overrun (or a worker panic, which drops the sender) emit the safe `unknown` and
/// exit 0, gracefully beating `BenchExec`'s SIGKILL. The watchdog only ever yields
/// UNKNOWN, so it can never produce an unsound verdict.
///
/// The budget's absolute deadline is also handed to the strategy, so a stage that
/// shells out to a long-running external tool can size its own timeout to fit inside
/// it rather than be killed with the task (see [`cpachecker_timelimit`]).''',
     '''/// Run compile -> ingest -> strategy on a worker thread under a wall-clock `budget`.
/// On overrun (or a worker panic, which drops the sender) emit the safe `unknown` and
/// exit 0, gracefully beating `BenchExec`'s SIGKILL. The watchdog only ever yields
/// UNKNOWN, so it can never produce an unsound verdict.
///
/// The budget is NOT threaded into the strategy: SAF shells out to no long-running
/// external tool (it bundles no SV-COMP participant), and every stage that can run
/// long -- fuzzing, native replay, concurrency shims -- carries its own deterministic
/// cost bound plus a local `Instant` safety valve.'''),

    ('''    let deadline = std::time::Instant::now() + budget;
    let (tx, rx) = std::sync::mpsc::channel();
    let _worker = std::thread::spawn(move || {
        let _ = tx.send(run_verdict(
            &input,
            data_model,
            property,
            specification,
            deadline,
        ));
    });''',
     '''    let (tx, rx) = std::sync::mpsc::channel();
    let _worker = std::thread::spawn(move || {
        let _ = tx.send(run_verdict(&input, data_model, property, specification));
    });'''),

    ('''fn run_verdict(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    property: saf_svcomp::Property,
    specification: String,
    deadline: std::time::Instant,
) -> VerdictOutcome {''',
     '''fn run_verdict(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    property: saf_svcomp::Property,
    specification: String,
) -> VerdictOutcome {'''),

    ('''        clang: &clang,
        deadline,
    };''',
     '''        clang: &clang,
    };'''),

    # --- the doc line part 3 got wrong --------------------------------------
    (None, None),  # placeholder, replaced below for SMOKE
]

SMOKE_FIX = (
    '''/// The TRUE arm abstains (doubly so now that it is disabled outright, but the
/// `-Winteger-overflow` probe fires independently); the UBSan FALSE path then
/// confirms the overflow.''',
    '''/// The TRUE arm is disabled outright, so it cannot yield `true` by any route; the
/// UBSan FALSE path then confirms the overflow. (Before the no-competitor-tools
/// decision this was guarded by a `-Winteger-overflow` probe plus the CPAchecker
/// gate; both are gone, and the assertion below is what still pins the behaviour.)''')


def drop_item(lines, name):
    """Delete a top-level fn/const by name, doc comment and attributes included."""
    pat = re.compile(r"^(?:pub )?(?:fn|const) " + re.escape(name) + r"\b")
    for i, l in enumerate(lines):
        if pat.match(l):
            s = i
            while s > 0 and (lines[s - 1].lstrip().startswith("///")
                             or lines[s - 1].lstrip().startswith("#[")):
                s -= 1
            if lines[i].rstrip().endswith(";"):
                return s, i
            e = i
            while e < len(lines) - 1:
                e += 1
                if lines[e] == "}":
                    return s, e
    return None


def main():
    text = CLI.read_text()
    if "bundles no SV-COMP participant" in text:
        sys.exit("already applied -- refusing to run twice")

    for old, new in EXACT:
        if old is None:
            continue
        if text.count(old) != 1:
            sys.exit(f"pattern appears {text.count(old)} times (want 1):\n---\n{old[:200]}")
        text = text.replace(old, new)
        print(f"  rewrote  {old.splitlines()[0][:72]}")

    lines = text.splitlines()
    edits = []
    for name in ["CPACHECKER_TIMELIMIT_MARGIN_SECS", "overflow_source_constant_folds"]:
        sp = drop_item(lines, name)
        if sp is None:
            sys.exit(f"could not locate {name}")
        edits.append(sp)
        print(f"  delete   {name:34s} lines {sp[0]+1}..{sp[1]+1}")
    edits.sort()
    for (s1, e1), (s2, _) in zip(edits, edits[1:]):
        if e1 >= s2:
            sys.exit("overlapping edits -- aborting")
    for s, e in reversed(edits):
        del lines[s:e + 1]
    CLI.write_text("\n".join(lines) + "\n")
    print(f"  -> {CLI}: now {len(lines)} lines")

    st = SMOKE.read_text()
    if st.count(SMOKE_FIX[0]) != 1:
        sys.exit("smoke.rs doc pattern not unique")
    SMOKE.write_text(st.replace(*SMOKE_FIX))
    print(f"  -> {SMOKE}: corrected the '-Winteger-overflow probe' claim")


if __name__ == "__main__":
    main()
