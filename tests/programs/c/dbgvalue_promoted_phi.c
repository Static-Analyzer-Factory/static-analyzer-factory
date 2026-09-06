// Fixture source for the dbg.value promoted-phi naming e2e test (plan 207 / 1b.0).
//
// Compiled with the `saf verify` recipe so the loop variables `s`/`i` survive
// ONLY as promoted SSA phis + llvm.dbg.value (dbg.declare is deleted by mem2reg):
//
//   clang-18 -g -S -emit-llvm -O0 -Xclang -disable-O0-optnone -Wno-everything -m32 \
//     dbgvalue_promoted_phi.c -o - \
//   | opt-18 -S -passes=mem2reg -o tests/fixtures/llvm/e2e/dbgvalue_promoted_phi.ll
//
// This is intentionally NOT the standard `clang -g -O0` (no mem2reg) fixture
// recipe, because the whole point of 1b.0 is naming values AFTER promotion.
int main(void) {
    int s = 0;
    for (int i = 0; i < 100; i++) {
        s += i;
    }
    return s;
}
