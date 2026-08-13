// R7 −32 regression guard: a global destructor loops forever AFTER main returns
// (compiles to `llvm.global_dtors`). The program never terminates. R7 must abstain
// ⇒ `unknown`, never `true`.
__attribute__((destructor)) void dtor(void) {
    while (1) {
    }
}

int main(void) {
    return 0;
}
