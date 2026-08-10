// unreach-call, expected verdict: FALSE. main() calls reach_error()
// unconditionally, so the error is trivially reachable.
extern void reach_error(void);

int main(void) {
    reach_error();
    return 0;
}
