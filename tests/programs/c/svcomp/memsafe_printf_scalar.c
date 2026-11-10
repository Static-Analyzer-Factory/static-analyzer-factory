/* printf with a constant format whose conversions read only scalar arguments
 * reads nothing but its own string literal: the program stays provable. */
extern int printf(const char *, ...);

int g;

int main(void) {
  g = 7;
  printf("g=%d hex=%#08x c=%c f=%5.2f p=%p %% %lu\n", g, g, 'a', 1.5, (void *)0,
         (unsigned long)g);
  return 0;
}
