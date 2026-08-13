// no-overflow FALSE: negation of INT_MIN (-INT_MIN is not representable) -> UBSan
// "negation of ... cannot be represented" -> false(no-overflow). Exercises the second
// of the three signed-overflow report forms.
#include <limits.h>
int main(void) {
    volatile int x = INT_MIN;
    return -x;
}
