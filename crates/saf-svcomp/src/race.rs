//! `no-data-race` FALSE finder + confirmer support (lever: `race-find`).
//!
//! This module implements the *propose* half of SAF's `propose → confirm →
//! witness` pipeline for the SV-COMP `no-data-race` property. It is an
//! Eraser/Goblint-style **over-approximate** lockset + may-happen-in-parallel
//! (MHP) race-candidate finder built on top of SAF's existing multi-threaded
//! analysis (`saf_analysis::mta`) and Andersen points-to analysis.
//!
//! # Soundness contract
//!
//! The finder is deliberately **over-approximate**: it reports a candidate
//! whenever two concurrent threads *may* access the same location with at least
//! one write and no *provably* common lock. It NEVER emits a verdict — the sole
//! soundness gate for a `false(no-data-race)` is the ThreadSanitizer native
//! replay in `saf-cli` (see the confirmer contract, R1/R7). Because the finder
//! only ever *gates* an expensive confirm run, a finder imprecision costs at
//! worst a wasted TSan run (over-report) or a missed recall opportunity
//! (under-report) — never an unsound verdict.
//!
//! Concretely the finder is used as follows: if it returns **zero** candidates,
//! the program is (per the over-approximation) race-free and the strategy
//! abstains without paying for a TSan run; if it returns **≥1** candidate, the
//! strategy compiles the ORIGINAL program under `-fsanitize=thread` and only
//! emits `false(no-data-race)` when TSan concretely observes a genuine data
//! race.
//!
//! # References (mechanism, not code)
//! - S. Savage et al., "Eraser: A Dynamic Data Race Detector for Multithreaded
//!   Programs" (SOSP'97) — the lockset discipline + virgin/exclusive/shared
//!   state machine that suppresses initialization-idiom false positives.
//! - Goblint's static lockset + MHP race analysis (TACAS SV-COMP concurrency).

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, InstId, LocId, ValueId};

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::mta::{MtaAnalysis, MtaConfig, ThreadId, compute_module_locksets};
use saf_analysis::{PtaConfig, PtaContext, PtaResult};
use std::sync::Arc;

/// A single over-approximate race candidate: two conflicting accesses that MAY
/// happen in parallel on a MAY-aliasing location without a provably common lock.
///
/// This is a *hint* for the confirmer and for diagnostics only; it carries no
/// verdict authority. Fields are kept coarse (function + instruction identity)
/// because the authoritative source locations for a witness come from the TSan
/// report, not from this static candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RaceCandidate {
    /// Thread id of the first access (0 = main).
    pub thread_a: u32,
    /// Thread id of the second access.
    pub thread_b: u32,
    /// Function containing the first access.
    pub func_a: FunctionId,
    /// Function containing the second access.
    pub func_b: FunctionId,
    /// Instruction of the first access.
    pub inst_a: InstId,
    /// Instruction of the second access.
    pub inst_b: InstId,
    /// Whether the first access writes.
    pub write_a: bool,
    /// Whether the second access writes.
    pub write_b: bool,
}

/// One thread's static access footprint: the load/store instructions reachable
/// from its entry function, with the mutex lock-identity set held at each.
struct ThreadAccess {
    thread_id: u32,
    func: FunctionId,
    inst: InstId,
    ptr: ValueId,
    write: bool,
    /// Canonical lock identity: the union of points-to `LocId`s of every mutex
    /// pointer held here. Two accesses share a lock iff these sets intersect.
    locks: BTreeSet<LocId>,
}

/// Find over-approximate `no-data-race` candidates for `module`.
///
/// Returns a deterministically-ordered list of candidate racing pairs. An empty
/// result means the finder could not construct any potential race (e.g. the
/// program has ≤1 thread, no shared writes, or every conflicting pair is
/// provably lock-protected) — the caller should then abstain. A non-empty result
/// means the caller should attempt a ThreadSanitizer confirmation.
#[must_use]
pub fn find_race_candidates(module: &AirModule) -> Vec<RaceCandidate> {
    // Build the analyses the finder needs. All are over-approximate and
    // deterministic (BTree-backed).
    let callgraph = CallGraph::build(module);
    let icfg = Icfg::build(module, &callgraph);

    let pta = {
        let mut ctx = PtaContext::new(PtaConfig::default());
        let raw = ctx.analyze(module);
        PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics)
    };

    let mta_config = MtaConfig::default();
    let mta = MtaAnalysis::new(module, &callgraph, &icfg, mta_config.clone()).analyze();

    // ≤1 thread ⇒ no concurrency ⇒ no data race possible.
    if mta.thread_graph.threads.len() <= 1 {
        return Vec::new();
    }

    let locksets = compute_module_locksets(module, &icfg, &mta_config);
    let accesses = collect_thread_accesses(module, &callgraph, &pta, &mta, &locksets);

    // Eraser-style state refinement: a location accessed by only ONE thread is
    // not shared, so it cannot race — suppress it. (This subsumes the
    // virgin/exclusive states of Eraser's machine for the purpose of candidate
    // generation: a location must reach the "shared" state — touched by ≥2
    // distinct threads — before it can be a candidate.) We approximate a
    // location by its points-to `LocId` set and mark, per `LocId`, the set of
    // threads that touch it.
    let mut loc_threads: BTreeMap<LocId, BTreeSet<u32>> = BTreeMap::new();
    for a in &accesses {
        if let Some(pts) = pta.points_to_ref(a.ptr) {
            for loc in pts {
                loc_threads.entry(*loc).or_default().insert(a.thread_id);
            }
        }
    }
    let shared: BTreeSet<LocId> = loc_threads
        .iter()
        .filter(|(_, ts)| ts.len() >= 2)
        .map(|(loc, _)| *loc)
        .collect();

    enumerate_candidates(&pta, &mta, &accesses, &shared)
}

/// Collect every thread's static load/store footprint, canonicalizing the
/// mutexes held at each access to their points-to `LocId`s.
fn collect_thread_accesses(
    module: &AirModule,
    callgraph: &CallGraph,
    pta: &PtaResult,
    mta: &saf_analysis::mta::MtaResult,
    locksets: &BTreeMap<FunctionId, saf_analysis::mta::LockSetResult>,
) -> Vec<ThreadAccess> {
    // A thread executes every function reachable (via the call graph) from its
    // entry function. A helper reachable from two thread entries is attributed to
    // BOTH, so races within a shared helper are found as cross-thread pairs.
    let mut accesses: Vec<ThreadAccess> = Vec::new();
    for (thread_id, tctx) in &mta.thread_graph.threads {
        let reachable = reachable_functions(callgraph, tctx.entry_function);
        for func in &module.functions {
            if func.is_declaration || !reachable.contains(&func.id) {
                continue;
            }
            let func_locks = locksets.get(&func.id);
            for block in &func.blocks {
                for inst in &block.instructions {
                    let (addr, write) = match &inst.op {
                        Operation::Load => (inst.operands.first().copied(), false),
                        Operation::Store => (inst.operands.get(1).copied(), true),
                        _ => continue,
                    };
                    let Some(addr) = addr else { continue };
                    // Canonicalize the held mutexes to their points-to LocId sets.
                    let mut locks = BTreeSet::new();
                    if let Some(lsr) = func_locks {
                        for m in lsr.lockset_at(inst.id).locks() {
                            if let Some(pts) = pta.points_to_ref(m) {
                                locks.extend(pts.iter().copied());
                            }
                        }
                    }
                    accesses.push(ThreadAccess {
                        thread_id: thread_id.0,
                        func: func.id,
                        inst: inst.id,
                        ptr: addr,
                        write,
                        locks,
                    });
                }
            }
        }
    }
    accesses
}

/// Enumerate conflicting cross-thread access pairs into deduplicated race
/// candidates. Deterministic: `accesses` is built in module/thread order and we
/// scan lower-triangular into a `BTreeSet`.
fn enumerate_candidates(
    pta: &PtaResult,
    mta: &saf_analysis::mta::MtaResult,
    accesses: &[ThreadAccess],
    shared: &BTreeSet<LocId>,
) -> Vec<RaceCandidate> {
    let touches_shared = |a: &ThreadAccess| -> bool {
        pta.points_to_ref(a.ptr)
            .is_some_and(|pts| pts.iter().any(|l| shared.contains(l)))
    };

    let mut candidates: BTreeSet<RaceCandidate> = BTreeSet::new();
    for (i, a) in accesses.iter().enumerate() {
        if !touches_shared(a) {
            continue;
        }
        for b in accesses.iter().skip(i + 1) {
            // A race needs two distinct threads that may run concurrently, with
            // at least one write and no provably common lock, on a MAY-aliasing
            // shared location.
            if a.thread_id == b.thread_id
                || !mta.may_run_concurrently(ThreadId(a.thread_id), ThreadId(b.thread_id))
                || (!a.write && !b.write)
                || !touches_shared(b)
                || !a.locks.is_disjoint(&b.locks)
                || !pta.may_alias(a.ptr, b.ptr).may_alias_conservative()
            {
                continue;
            }
            let (a, b) = order_pair(a, b);
            candidates.insert(RaceCandidate {
                thread_a: a.thread_id,
                thread_b: b.thread_id,
                func_a: a.func,
                func_b: b.func,
                inst_a: a.inst,
                inst_b: b.inst,
                write_a: a.write,
                write_b: b.write,
            });
        }
    }
    candidates.into_iter().collect()
}

/// Canonical ordering of a candidate pair so that `(a,b)` and `(b,a)` dedup to
/// one entry.
fn order_pair<'x>(
    a: &'x ThreadAccess,
    b: &'x ThreadAccess,
) -> (&'x ThreadAccess, &'x ThreadAccess) {
    if (a.thread_id, a.inst) <= (b.thread_id, b.inst) {
        (a, b)
    } else {
        (b, a)
    }
}

/// Compute the set of functions reachable from `entry` via the call graph
/// (direct edges only; indirect placeholders are ignored — an over-approximate
/// caller that misses an indirect edge only loses finder recall, never
/// soundness, since TSan is the arbiter).
fn reachable_functions(cg: &CallGraph, entry: FunctionId) -> BTreeSet<FunctionId> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![entry];
    seen.insert(entry);
    while let Some(f) = stack.pop() {
        let Some(node) = cg.node_for_function(f) else {
            continue;
        };
        let Some(callees) = cg.callees_of(node) else {
            continue;
        };
        for callee in callees {
            if let Some(cf) = callee.function_id() {
                if seen.insert(cf) {
                    stack.push(cf);
                }
            }
        }
    }
    seen
}

// ---------------------------------------------------------------------------
// TSan report parsing (the confirmer's arbiter — R1)
// ---------------------------------------------------------------------------

/// A confirmed ThreadSanitizer data race, extracted from a TSan report.
///
/// Only produced for a genuine `WARNING: ThreadSanitizer: data race` — never for
/// lock-order-inversion, thread leaks, signal-unsafe calls, or any other TSan
/// diagnostic (R1: confirm ONLY on the property's exact violation event).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaceHit {
    /// The `SUMMARY` source location `file:line`, if present (best-effort).
    pub summary_location: Option<String>,
    /// Source `file:line` of the first (later) racing access, if present.
    pub access_location: Option<String>,
}

/// Parse a ThreadSanitizer stderr report, returning a [`RaceHit`] IFF it
/// contains a genuine data-race warning.
///
/// R1 fail-closed: any other TSan diagnostic (deadlock, thread leak,
/// signal-unsafe, mutex misuse, use-after-free) returns `None` (abstain). This
/// is the sole event that confirms a `no-data-race` FALSE.
#[must_use]
pub fn parse_tsan_report(stderr: &str) -> Option<RaceHit> {
    // The exact TSan banner for a data race. TSan uses this precise phrasing;
    // other reports read e.g. "ThreadSanitizer: lock-order-inversion",
    // "ThreadSanitizer: thread leak", "ThreadSanitizer: data race on vptr"
    // (still a data race — accepted), etc. We require the literal "data race"
    // banner and reject every other TSan diagnostic (R1).
    let has_data_race = stderr.lines().any(|l| {
        l.trim_start()
            .starts_with("WARNING: ThreadSanitizer: data race")
    });
    if !has_data_race {
        return None;
    }

    // Best-effort source-location extraction for a witness hint. The SUMMARY
    // line reads: "SUMMARY: ThreadSanitizer: data race <file>:<line> in <fn>".
    let summary_location = stderr
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix("SUMMARY: ThreadSanitizer: data race ")
        })
        .map(|rest| rest.split_whitespace().next().unwrap_or(rest).to_string());

    // The first stack frame after the "Write of size ..."/"Read of size ..."
    // line carries the racing access location: "    #0 fn <file>:<line> (...)".
    let access_location = extract_first_frame_location(stderr);

    Some(RaceHit {
        summary_location,
        access_location,
    })
}

/// Extract the `file:line` from the first `#0` stack frame in a TSan report.
fn extract_first_frame_location(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("#0 ") {
            // "<fn> <file>:<line> (<module>+<off>)" — the file:line is the token
            // containing a ':' with a trailing line number.
            for tok in rest.split_whitespace() {
                if let Some((_file, line_no)) = tok.rsplit_once(':') {
                    if line_no.chars().all(|c| c.is_ascii_digit()) && !line_no.is_empty() {
                        return Some(tok.to_string());
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// GraphML 1.0 violation witness (R7: concurrency FALSE needs GraphML, not YAML)
// ---------------------------------------------------------------------------

/// A fixed, deterministic creation timestamp. SV-COMP requires an ISO-8601
/// `creationtime`; determinism (byte-identical witnesses for identical inputs)
/// forbids reading the wall clock, so we emit a constant. Validators do not
/// verify the timestamp's freshness.
const WITNESS_CREATION_TIME: &str = "1970-01-01T00:00:00Z";

/// Escape a string for inclusion in XML character data / attribute values.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Build a GraphML 1.0 violation witness for a confirmed `no-data-race` FALSE.
///
/// Per the confirmer contract R7, a concurrency-property violation witness MUST
/// be GraphML 1.0 (a YAML-2.0 witness scores 0 for concurrency). This emits a
/// structurally-valid SV-COMP violation witness: the mandatory graph metadata
/// (`witness-type`, `sourcecodelang`, `producer`, `specification`,
/// `programfile`, `programhash`, `architecture`, `creationtime`) plus an entry
/// node, a `createThread` edge modelling the spawn, and a `threadId`-annotated
/// edge into the `violation` node. Targeted at CPAchecker / Dartagnan /
/// `ConcurrentWitness2Test`.
///
/// The output is fully deterministic given its inputs.
#[must_use]
pub fn race_graphml_witness(
    spec: &str,
    programfile: &str,
    programhash: &str,
    architecture: &str,
    hit: &RaceHit,
) -> String {
    use std::fmt::Write as _;

    let startline = hit
        .access_location
        .as_deref()
        .or(hit.summary_location.as_deref())
        .and_then(|loc| loc.rsplit_once(':').map(|(_, l)| l.to_string()))
        .filter(|l| l.chars().all(|c| c.is_ascii_digit()) && !l.is_empty());

    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n");
    s.push_str(
        "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" \
xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n",
    );
    // Graph-level keys.
    for (id, ty) in [
        ("witness-type", "string"),
        ("sourcecodelang", "string"),
        ("producer", "string"),
        ("specification", "string"),
        ("programfile", "string"),
        ("programhash", "string"),
        ("architecture", "string"),
        ("creationtime", "string"),
    ] {
        let _ = writeln!(
            s,
            "  <key attr.name=\"{id}\" attr.type=\"{ty}\" for=\"graph\" id=\"{id}\"/>"
        );
    }
    // Node keys.
    s.push_str(
        "  <key attr.name=\"entry\" attr.type=\"boolean\" for=\"node\" id=\"entry\">\
<default>false</default></key>\n",
    );
    s.push_str(
        "  <key attr.name=\"violation\" attr.type=\"boolean\" for=\"node\" id=\"violation\">\
<default>false</default></key>\n",
    );
    // Edge keys.
    s.push_str(
        "  <key attr.name=\"threadId\" attr.type=\"string\" for=\"edge\" id=\"threadId\"/>\n",
    );
    s.push_str(
        "  <key attr.name=\"createThread\" attr.type=\"string\" for=\"edge\" id=\"createThread\"/>\n",
    );
    s.push_str(
        "  <key attr.name=\"startline\" attr.type=\"int\" for=\"edge\" id=\"startline\"/>\n",
    );

    s.push_str("  <graph edgedefault=\"directed\">\n");
    for (k, v) in [
        ("witness-type", "violation_witness"),
        ("sourcecodelang", "C"),
        ("producer", "SAF"),
        ("specification", spec),
        ("programfile", programfile),
        ("programhash", programhash),
        ("architecture", architecture),
        ("creationtime", WITNESS_CREATION_TIME),
    ] {
        let _ = writeln!(s, "    <data key=\"{k}\">{}</data>", xml_escape(v));
    }
    // Entry node (thread 0 = main), the spawned thread, and the violation node.
    s.push_str("    <node id=\"N0\"><data key=\"entry\">true</data></node>\n");
    s.push_str("    <node id=\"N1\"/>\n");
    s.push_str("    <node id=\"N2\"><data key=\"violation\">true</data></node>\n");
    // main spawns the racing thread (createThread models pthread_create).
    s.push_str(
        "    <edge source=\"N0\" target=\"N1\"><data key=\"createThread\">1</data></edge>\n",
    );
    // The racing access runs in the spawned thread and reaches the violation.
    s.push_str("    <edge source=\"N1\" target=\"N2\"><data key=\"threadId\">1</data>");
    if let Some(line) = &startline {
        let _ = write!(s, "<data key=\"startline\">{line}</data>");
    }
    s.push_str("</edge>\n");
    s.push_str("  </graph>\n");
    s.push_str("</graphml>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphml_witness_is_well_formed() {
        let hit = RaceHit {
            summary_location: Some("/tmp/race.c:8".to_string()),
            access_location: Some("/tmp/race.c:8".to_string()),
        };
        let w = race_graphml_witness(
            "CHECK( init(main()), LTL(G ! data-race) )",
            "race.c",
            "abc123",
            "64bit",
            &hit,
        );
        assert!(w.starts_with("<?xml"));
        assert!(w.contains("violation_witness"));
        assert!(w.contains("<data key=\"violation\">true</data>"));
        assert!(w.contains("createThread"));
        assert!(w.contains("<data key=\"threadId\">1</data>"));
        assert!(w.contains("<data key=\"startline\">8</data>"));
        assert!(w.trim_end().ends_with("</graphml>"));
        // Deterministic: same inputs → identical bytes.
        let w2 = race_graphml_witness(
            "CHECK( init(main()), LTL(G ! data-race) )",
            "race.c",
            "abc123",
            "64bit",
            &hit,
        );
        assert_eq!(w, w2);
    }

    #[test]
    fn graphml_witness_escapes_xml() {
        let hit = RaceHit {
            summary_location: None,
            access_location: None,
        };
        let w = race_graphml_witness("a & b < c > d \"e\"", "p.c", "h", "32bit", &hit);
        assert!(w.contains("a &amp; b &lt; c &gt; d &quot;e&quot;"));
        // No `startline` DATA element when no location is known (the key
        // definition is always present, but no value is emitted).
        assert!(!w.contains("<data key=\"startline\">"));
    }

    #[test]
    fn parse_rejects_non_race_reports() {
        assert!(parse_tsan_report("").is_none());
        assert!(parse_tsan_report("all good, no sanitizer output").is_none());
        // Deadlock / lock-order-inversion is NOT a data race.
        let loi = "WARNING: ThreadSanitizer: lock-order-inversion (potential deadlock) (pid=1)\n";
        assert!(parse_tsan_report(loi).is_none());
        // Thread leak is NOT a data race.
        let leak = "WARNING: ThreadSanitizer: thread leak (pid=1)\n";
        assert!(parse_tsan_report(leak).is_none());
        // Signal-unsafe call is NOT a data race.
        let sig = "WARNING: ThreadSanitizer: signal-unsafe call inside of a signal\n";
        assert!(parse_tsan_report(sig).is_none());
    }

    #[test]
    fn parse_accepts_genuine_data_race() {
        let report = "\
==================
WARNING: ThreadSanitizer: data race (pid=12345)
  Write of size 4 at 0x7b0400000010 by thread T1:
    #0 worker /tmp/race.c:8 (a.out+0x123)
  Previous read of size 4 at 0x7b0400000010 by main thread:
    #0 main /tmp/race.c:20 (a.out+0x456)
  Location is global 'shared' of size 4 at 0x7b0400000010 (a.out+0x789)
SUMMARY: ThreadSanitizer: data race /tmp/race.c:8 in worker
==================
";
        let hit = parse_tsan_report(report).expect("should confirm a data race");
        assert_eq!(hit.summary_location.as_deref(), Some("/tmp/race.c:8"));
        assert_eq!(hit.access_location.as_deref(), Some("/tmp/race.c:8"));
    }

    #[test]
    fn parse_accepts_data_race_on_vptr() {
        // "data race on vptr (ctor/dtor vs virtual call)" still starts with the
        // data-race banner and is a genuine race.
        let report =
            "WARNING: ThreadSanitizer: data race on vptr (ctor/dtor vs virtual call) (pid=1)\n";
        assert!(parse_tsan_report(report).is_some());
    }

    #[test]
    fn find_candidates_empty_for_non_threaded_module() {
        use saf_core::id::make_id;
        use saf_core::ids::ModuleId;
        // An empty module has ≤1 thread ⇒ no candidates.
        let module = AirModule {
            id: ModuleId(make_id("test_mod", b"race")),
            name: Some("test".to_string()),
            functions: Vec::new(),
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: BTreeMap::new(),
            types: BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        assert!(find_race_candidates(&module).is_empty());
    }
}
