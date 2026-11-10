/* The format lives in a mutable global that the program rewrites to "%s"
 * before the call, through one anchored 2-byte store (no GEP, no cast
 * instruction under opaque pointers), so every access the program makes itself
 * is a safe whole-object global access. printf then walks the unterminated
 * `msg`. Only a constant string literal may be read as the format. */
extern int printf(const char *, ...);

char fmt[3] = {'%', 'd', 0};
char msg[4] = {'a', 'b', 'c', 'd'}; /* deliberately NOT NUL-terminated */

int main(void) {
  *(short *)fmt = 0x7325; /* "%s" on a little-endian target */
  printf(fmt, msg);
  return 0;
}
