#include <stdlib.h>
// getenv → helper → system (across function boundary)
char *process(char *input) {
    return input;  // pass-through
}
int main() {
    char *data = getenv("USER");
    char *result = process(data);
    system(result);
    return 0;
}
