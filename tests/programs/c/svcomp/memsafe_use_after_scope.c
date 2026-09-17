/* plans/214 Movement 2 -- pins that stack anchoring does NOT leak across scope.
 *
 * Allowing an alloca to anchor is only sound because a DIRECT access to the
 * slot's own `ValueId` can occur solely inside the function whose frame owns it.
 * The dangerous shape is this one: the address ESCAPES its block through a
 * pointer variable, and clang -- which hoists every block-scoped local to a
 * function-entry alloca and emits no lifetime intrinsic -- leaves nothing in the
 * IR to say `y`'s scope ended.
 *
 * TWO independent rules reject it, and the reported reason is the first one the
 * scan reaches, not the interesting one:
 *   1. `store-width-unknown` -- storing `&y` into `p` is a POINTER-width store,
 *      and `exact_size_of` refuses pointer widths because the AIR's
 *      `target_pointer_width` is hardcoded to 8 while these tasks are ILP32.
 *   2. had it got past that, `*p = 1` dereferences a value that came out of
 *      MEMORY, and a load result never anchors.
 *
 * Rule 1 makes rule 2 unreachable for any program that moves a pointer through
 * memory at all, which is worth knowing: the anchoring rule is belt-and-braces
 * here rather than the thing doing the work. Assert only that it abstains. */
int *p;

int main(void) {
  {
    int y = 0;
    p = &y;
  }
  *p = 1;
  return 0;
}
