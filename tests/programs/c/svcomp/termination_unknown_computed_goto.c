// R7 −32 regression guard: an infinite loop via computed goto (GCC
// labels-as-values → LLVM `indirectbr`, which the frontend drops). The block ends
// with no recognized terminator; the CFG-completeness gate makes R7 abstain ⇒
// `unknown`, never `true` (else the back-edge would be invisible = −32).
int main(void) {
    void *l = &&spin;
spin:
    goto *l;
}
