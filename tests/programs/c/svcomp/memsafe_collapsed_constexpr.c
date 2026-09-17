/* plans/214 Movement 2 -- pins the INGEST-FIDELITY gate.
 *
 * `arr` is 400 bytes and the store lands 2000 bytes past its base. Measured at
 * 5f274d13: this program and the same one with `arr[5]` produce STRUCTURALLY
 * IDENTICAL AIR. Clang folds the index into a constant-expression
 * `getelementptr`, and ingestion resolves that expression to `arr`'s bare
 * `ValueId` -- `FieldPath` has no way to express "N bytes past the object", so
 * the offset is simply gone. The prover sees an anchored, offset-0, 4-byte store
 * into a 400-byte object and every obligation is satisfied.
 *
 * Nothing in the instruction stream can catch this, which is why
 * `IngestFidelity::collapsed_const_ptr_expr` exists. Drop that flag and this
 * fixture becomes a wrong TRUE worth -32, uncapped. */
int arr[100];

int main(void) {
  arr[500] = 1;
  return arr[500];
}
