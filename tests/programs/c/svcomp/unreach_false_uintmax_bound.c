// plans/213 §2b, the `loop-simple/deep-nested` shape reduced to one comparison.
// `0xfffffffe` is `-2` in the signless domain, so signed refinement of
// `[0,0] < [-2,-2]` is bottom and the body -- which really is reachable, since
// `0 <u 0xfffffffe` is TRUE -- was declared dead.
extern void abort(void);
void reach_error() {}

int main() {
  unsigned int a = 0;
  unsigned int uint32_max = 0xffffffff;

  if (a < uint32_max - 1) {
    reach_error();
    abort();
  }
  return 0;
}
