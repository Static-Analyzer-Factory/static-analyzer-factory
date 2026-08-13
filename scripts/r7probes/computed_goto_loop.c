// R7 −32 probe: an infinite loop via computed goto (GCC labels-as-values →
// LLVM `indirectbr`). If `Cfg::build`/`extract_successors` does not model
// `indirectbr` edges, the back-edge is invisible and R7 wrongly calls this
// loop-free → `true` = a −32 (SV-COMP termination verdict = false).
int main(void) {
    void *l = &&spin;
spin:
    goto *l;
}
