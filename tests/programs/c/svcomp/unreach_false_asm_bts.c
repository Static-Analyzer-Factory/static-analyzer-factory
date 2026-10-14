// Inline asm that writes memory (Linux set_bit's bts): the frontend drops the asm, so
// the TRUE provers must refuse the module; reach_error is reachable.
extern void reach_error(void);
unsigned long flags = 0;
int main(void) {
  __asm__ volatile("btsq %1,%0" : "+m"(flags) : "Ir"(3L) : "memory");
  if (flags == 8) reach_error();
  return 0;
}
