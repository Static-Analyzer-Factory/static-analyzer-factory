// R7 −32 probe: a global constructor loops forever BEFORE main. The program
// never terminates, but the loop lives in `ctor`, which is NOT reachable from
// `main` via the call graph. If R7 emits `true`, that is a −32 soundness bug
// (SV-COMP termination verdict = false / non-terminating).
__attribute__((constructor)) void ctor(void) {
    while (1) {
    }
}

int main(void) {
    return 0;
}
