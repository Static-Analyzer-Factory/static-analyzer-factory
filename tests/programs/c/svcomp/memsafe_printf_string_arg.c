/* printf's %s walks `msg` until a NUL, and `msg` has none: the out-of-bounds
 * read happens inside libc. A %s conversion must keep printf non-inert. */
extern int printf(const char *, ...);

char msg[4] = {'a', 'b', 'c', 'd'}; /* deliberately NOT NUL-terminated */

int main(void) {
  printf("%s\n", msg);
  return 0;
}
