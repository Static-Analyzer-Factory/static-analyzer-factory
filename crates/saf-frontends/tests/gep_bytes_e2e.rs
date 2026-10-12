//! The LLVM frontend records, for a consumer that addresses memory by byte
//! offset, the byte layout of every global (`AirGlobal::layout`) and the byte
//! arithmetic of every `getelementptr` step (`GEP_BYTES_KEY`): plan 222 §2q,
//! after a GEP's structural path (relative to the GEP's own source element
//! type) mis-addressed a `[3 x [2 x float]]` global read through a decayed
//! row pointer.

use std::path::PathBuf;

use saf_core::air::{AirBundle, GEP_BYTES_KEY, Layout, LayoutField, Operation};
use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn load(name: &str) -> AirBundle {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/llvm/e2e")
        .join(name);
    LlvmFrontend::new()
        .ingest(&[path.as_path()], &Config::default())
        .unwrap_or_else(|e| panic!("Failed to load fixture '{name}': {e}"))
}

/// The `[kind, n]` pairs of every GEP in `func`, in instruction order.
fn gep_bytes(bundle: &AirBundle, func: &str) -> Vec<Option<Vec<(u64, u64)>>> {
    let f = bundle
        .module
        .functions
        .iter()
        .find(|f| f.name == func)
        .unwrap_or_else(|| panic!("no function {func}"));
    f.blocks
        .iter()
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i.op, Operation::Gep { .. }))
        .map(|i| {
            i.extensions.get(GEP_BYTES_KEY).map(|v| {
                v.as_array()
                    .expect("array")
                    .iter()
                    .map(|p| {
                        let p = p.as_array().expect("pair");
                        (p[0].as_u64().unwrap(), p[1].as_u64().unwrap())
                    })
                    .collect()
            })
        })
        .collect()
}

#[test]
fn globals_carry_their_byte_layout() {
    let bundle = load("gep_bytes_rows.ll");
    let layout = |name: &str| {
        bundle
            .module
            .globals
            .iter()
            .find(|g| g.name == name)
            .unwrap_or_else(|| panic!("no global {name}"))
            .layout
            .clone()
            .unwrap_or_else(|| panic!("{name} has no layout"))
    };
    let f32 = || Box::new(Layout::Scalar { size: 4 });
    assert_eq!(
        layout("W"),
        Layout::Array {
            count: 3,
            stride: 8,
            element: Box::new(Layout::Array {
                count: 2,
                stride: 4,
                element: f32(),
            }),
        }
    );
    // `{ i8, i32, [2 x i16] }`: the i32 is padded to offset 4, the array
    // follows at 8, and the struct is 12 bytes.
    assert_eq!(
        layout("s"),
        Layout::Struct {
            size: 12,
            fields: vec![
                LayoutField {
                    offset: 0,
                    layout: Layout::Scalar { size: 1 },
                },
                LayoutField {
                    offset: 4,
                    layout: Layout::Scalar { size: 4 },
                },
                LayoutField {
                    offset: 8,
                    layout: Layout::Array {
                        count: 2,
                        stride: 2,
                        element: Box::new(Layout::Scalar { size: 2 }),
                    },
                },
            ],
        }
    );
    let w = layout("W");
    assert_eq!(
        w.scalar_at(4),
        Some((vec![0, 1], 4)),
        "W[0][1] starts at byte 4"
    );
    assert_eq!(
        w.scalar_at(8),
        Some((vec![1, 0], 4)),
        "W[1][0] starts at byte 8"
    );
    assert_eq!(w.scalar_at(6), None, "inside a float");
    assert_eq!(w.offset_of_path(&[2, 1]), Some(20));
    assert_eq!(layout("s").scalar_at(10), Some((vec![2, 1], 2)));
}

#[test]
fn geps_carry_their_byte_arithmetic() {
    let bundle = load("gep_bytes_rows.ll");
    // `[2 x float], ptr %p, i32 %c`: one pointer-level step of 8 bytes;
    // `[2 x float], ptr %row, i32 0, i32 %i`: 8-byte step then 4-byte elements.
    assert_eq!(
        gep_bytes(&bundle, "get"),
        vec![Some(vec![(0, 8)]), Some(vec![(0, 8), (0, 4)])]
    );
    // `%struct.S, ptr @s, i32 0, i32 2, i32 1`: 12-byte step, field 2 at
    // byte 8, then 2-byte elements.
    assert_eq!(
        gep_bytes(&bundle, "main"),
        vec![Some(vec![(0, 12), (1, 8), (0, 2)])]
    );
}
