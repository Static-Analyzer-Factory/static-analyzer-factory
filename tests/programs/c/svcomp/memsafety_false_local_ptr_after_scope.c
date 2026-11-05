/* A LOCAL pointer variable keeps the address of a block-scoped object past
   the block; mem2reg turns `*p` into a direct access to the dead slot.
   Expected: false(valid-deref); never true. */
int main(void) {
  int *p;
  {
    int y = 0;
    p = &y;
  }
  *p = 1;
  return 0;
}
