/* %n makes printf WRITE through a pointer argument: here 4 bytes into a 1-byte
 * global. printf with %n must stay non-inert. */
extern int printf(const char *, ...);

char small;

int main(void) {
  printf("abc%n\n", (int *)&small);
  return 0;
}
