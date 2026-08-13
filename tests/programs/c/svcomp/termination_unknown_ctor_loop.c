// R7 −32 regression guard: a global constructor loops forever BEFORE main. The
// program never terminates, but the loop lives outside main's call graph. R7 must
// abstain (the module has `llvm.global_ctors`) ⇒ `unknown`, never `true`.
__attribute__((constructor)) void ctor(void) {
    while (1) {
    }
}

int main(void) {
    return 0;
}
