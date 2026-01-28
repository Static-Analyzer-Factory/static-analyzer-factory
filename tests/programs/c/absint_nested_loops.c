// Abstract interpretation test: nested loops with matrix access
// Tests widening precision at multiple loop headers.

#include <stdlib.h>

#define ROWS 8
#define COLS 8

void init_matrix(int matrix[ROWS][COLS]) {
    for (int i = 0; i < ROWS; i++) {
        for (int j = 0; j < COLS; j++) {
            matrix[i][j] = i * COLS + j;
        }
    }
}

int sum_matrix(int matrix[ROWS][COLS]) {
    int total = 0;
    for (int i = 0; i < ROWS; i++) {
        for (int j = 0; j < COLS; j++) {
            total = total + matrix[i][j];
        }
    }
    return total;
}

int main(void) {
    int matrix[ROWS][COLS];
    init_matrix(matrix);
    return sum_matrix(matrix);
}
