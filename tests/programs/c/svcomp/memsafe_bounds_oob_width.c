/* plans/214 Movement 2 -- pins OBLIGATION 3 (bounds), and nothing else.
 *
 * `g` is one byte; the store writes four. Note what is NOT here: under LLVM 18's
 * opaque pointers a pointer-to-pointer cast is a no-op, so `(int *)&g` leaves NO
 * cast instruction and NO constant expression in the IR. The AIR is a plain
 * `Store` of an `i32` whose address operand IS the global -- indistinguishable
 * from a safe whole-object store except by comparing the access WIDTH against
 * the object's SIZE.
 *
 * So this fixture fails (PROVEs a 3-byte overflow) the moment `check_access`
 * stops checking `width <= size_of(global)`, which is the only thing standing
 * between it and a wrong TRUE. It is also why `AirGlobal.value_type` had to be
 * populated: without it there is no size to compare against. */
char g;

int main(void) {
  *(int *)&g = 1;
  return 0;
}
