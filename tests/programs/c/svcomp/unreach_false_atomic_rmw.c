// An atomic read-modify-write in a sequential program: `x` becomes 7, so
// reach_error IS reachable. The frontend models `atomicrmw` as storing its
// operand (2), which made the TRUE provers read `x == 7` as impossible.
extern void abort(void);
extern void __assert_fail(const char *, const char *, unsigned int, const char *);
void reach_error(void) { __assert_fail("0", "unreach_false_atomic_rmw.c", 3, "reach_error"); }
int x = 5;
int main(void) {
  __sync_fetch_and_add(&x, 2);
  if (x == 7) {
    reach_error();
    abort();
  }
  return 0;
}
