// no-overflow FALSE: INT_MIN / -1 (the quotient INT_MAX+1 is not representable) ->
// UBSan "division of ... by -1 cannot be represented" -> false(no-overflow). Exercises
// the third signed-overflow report form (NOT divide-by-zero, which is a different,
// excluded UBSan check).
#include <limits.h>
int main(void) {
    volatile int x = INT_MIN;
    volatile int y = -1;
    return x / y;
}
