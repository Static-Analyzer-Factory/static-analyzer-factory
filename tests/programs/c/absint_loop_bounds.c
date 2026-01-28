// Abstract interpretation test: loop bound analysis
// Tests that the fixpoint iterator correctly tracks loop counters
// and can determine array access safety via widening thresholds.

#include <stdlib.h>

#define ARRAY_SIZE 100

void init_array(int *arr, int size) {
    // Loop counter i ∈ [0, size-1]
    for (int i = 0; i < size; i++) {
        arr[i] = 0;
    }
}

int sum_array(int *arr, int size) {
    int total = 0;
    for (int i = 0; i < size; i++) {
        total = total + arr[i];
    }
    return total;
}

int count_positive(int *arr, int size) {
    int count = 0;
    for (int i = 0; i < size; i++) {
        if (arr[i] > 0) {
            count = count + 1;
        }
    }
    return count;
}

int main(void) {
    int data[ARRAY_SIZE];
    init_array(data, ARRAY_SIZE);
    int s = sum_array(data, ARRAY_SIZE);
    int c = count_positive(data, ARRAY_SIZE);
    return s + c;
}
