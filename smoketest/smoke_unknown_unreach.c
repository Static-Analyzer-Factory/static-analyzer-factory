// SV-COMP smoke fixture -- property: unreach-call, expected verdict: UNKNOWN.
//
// `saf_smoke_opaque()` is declared but never defined, so the guard's value is
// undecidable by construction: no sound analysis can prove the call to
// `reach_error()` unreachable (the callee's semantics are unknown), and no
// concrete replay can refute it (the harness would not link). SAF must
// therefore abstain.
//
// The fixture is deliberately undecidable rather than merely "hard": a fixture
// that is genuinely TRUE or genuinely FALSE would start producing a different
// verdict the moment SAF's prover or refuter improves, silently breaking the
// archive's own smoke test. This one stays `unknown` for every sound version.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);
extern int saf_smoke_opaque(int);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (saf_smoke_opaque(x) == 42) {
        reach_error();
    }
    return 0;
}
