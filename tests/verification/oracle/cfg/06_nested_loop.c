// PURPOSE: Nested loops create two loop headers
void test() {
    for (int i = 0; i < 10; i++) {
        for (int j = 0; j < 10; j++) {
            // inner work
        }
    }
}
