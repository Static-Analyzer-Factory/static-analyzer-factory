// Wrong-TRUE guard (frontend/engine review 2026-10-14): reach_error is reachable.
extern void reach_error(void);
int g;
int main(void) {
  __asm__ volatile("movl $1, g(%%rip)" ::: "memory");
  if (g) reach_error();
  return 0;
}
