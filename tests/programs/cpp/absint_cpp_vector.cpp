// Abstract interpretation test: C++ struct with size/capacity
// Tests interval tracking through struct field access patterns.

#include <cstdlib>

struct SimpleVector {
    int* data;
    int size;
    int capacity;
};

SimpleVector* vec_create(int cap) {
    SimpleVector* v = (SimpleVector*)malloc(sizeof(SimpleVector));
    if (!v) return nullptr;
    v->data = (int*)malloc(cap * sizeof(int));
    v->size = 0;
    v->capacity = cap;
    return v;
}

void vec_push(SimpleVector* v, int value) {
    if (v->size < v->capacity) {
        v->data[v->size] = value;  // Safe: size < capacity
        v->size = v->size + 1;
    }
    // No reallocation — just drop if full
}

int vec_get(SimpleVector* v, int index) {
    // No bounds check — potential OOB if index >= size
    return v->data[index];
}

int vec_get_safe(SimpleVector* v, int index) {
    if (index >= 0 && index < v->size) {
        return v->data[index];  // Safe: bounds checked
    }
    return -1;
}

void vec_free(SimpleVector* v) {
    if (v) {
        free(v->data);
        free(v);
    }
}

int main() {
    SimpleVector* v = vec_create(16);
    if (!v) return 1;

    for (int i = 0; i < 10; i++) {
        vec_push(v, i * 3);
    }

    int val1 = vec_get_safe(v, 5);   // Safe
    int val2 = vec_get(v, 20);       // OOB: 20 >= size(10)

    vec_free(v);
    return val1 + val2;
}
