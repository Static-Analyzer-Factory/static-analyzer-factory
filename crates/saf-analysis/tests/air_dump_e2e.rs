//! Dump AIR JSON from the LLVM frontend for comparison with tree-sitter converter.
//!
//! Run: `cargo test -p saf-analysis --test air_dump_e2e -- --nocapture`
//!
//! Outputs normalized AIR structure to stdout for each fixture, including:
//! - Function names, param counts, block counts
//! - Per-block instruction opcodes, operand counts, field paths
//! - Global variable names, initializer kinds

use saf_core::air::{AirBundle, Operation};
use saf_test_utils::load_ll_bundle;

fn print_air_summary(name: &str, bundle: &AirBundle) {
    let module = &bundle.module;
    println!("\n=== {name} ===");
    println!("Functions: {}", module.functions.len());

    for func in &module.functions {
        let decl = if func.is_declaration {
            " (declaration)"
        } else {
            ""
        };
        println!(
            "  fn {}{}: {} params, {} blocks",
            func.name,
            decl,
            func.params.len(),
            func.blocks.len()
        );

        for (bi, block) in func.blocks.iter().enumerate() {
            let label = block.label.as_deref().unwrap_or("(none)");
            println!(
                "    block[{}] \"{}\": {} instructions",
                bi,
                label,
                block.instructions.len()
            );

            for (ii, inst) in block.instructions.iter().enumerate() {
                let op_name = op_to_str(&inst.op);
                let has_dst = if inst.dst.is_some() {
                    "dst=yes"
                } else {
                    "dst=no"
                };
                let operands = inst.operands.len();
                let mut extras = Vec::new();

                match &inst.op {
                    Operation::BinaryOp { kind } => extras.push(format!("kind={kind:?}")),
                    Operation::Cast { kind, .. } => extras.push(format!("kind={kind:?}")),
                    Operation::Gep { field_path } if !field_path.steps.is_empty() => {
                        let steps: Vec<String> =
                            field_path.steps.iter().map(|s| format!("{s:?}")).collect();
                        extras.push(format!("field_path=[{}]", steps.join(", ")));
                    }
                    Operation::Alloca { size_bytes } => {
                        if let Some(sz) = size_bytes {
                            extras.push(format!("size_bytes={sz}"));
                        }
                    }
                    Operation::HeapAlloc { kind } => extras.push(format!("kind={kind:?}")),
                    Operation::CallDirect { callee } => {
                        extras.push(format!("callee={}", callee.to_hex()));
                    }
                    _ => {}
                }

                let extra_str = if extras.is_empty() {
                    String::new()
                } else {
                    format!(" {}", extras.join(" "))
                };

                println!("      inst[{ii}] {op_name} operands={operands} {has_dst}{extra_str}");
            }
        }
    }

    println!("Globals: {}", module.globals.len());
    for global in &module.globals {
        let constant = if global.is_constant {
            " (constant)"
        } else {
            ""
        };
        let init = global
            .init
            .as_ref()
            .map_or_else(|| "none".to_string(), |c| format!("{c:?}"));
        println!("  global {}{}: init={}", global.name, constant, init);
    }

    // Constants map
    if !module.constants.is_empty() {
        println!("Constants: {}", module.constants.len());
        for (vid, constant) in &module.constants {
            println!("  {} = {:?}", vid.to_hex(), constant);
        }
    }
}

fn op_to_str(op: &Operation) -> &'static str {
    match op {
        Operation::Alloca { .. } => "alloca",
        Operation::Load => "load",
        Operation::Store => "store",
        Operation::Gep { .. } => "gep",
        Operation::Phi { .. } => "phi",
        Operation::Select => "select",
        Operation::BinaryOp { .. } => "binary_op",
        Operation::Cast { .. } => "cast",
        Operation::CallDirect { .. } => "call_direct",
        Operation::CallIndirect { .. } => "call_indirect",
        Operation::Ret => "ret",
        Operation::Br { .. } => "br",
        Operation::CondBr { .. } => "cond_br",
        Operation::Switch { .. } => "switch",
        Operation::Unreachable => "unreachable",
        Operation::Copy => "copy",
        Operation::Freeze => "freeze",
        Operation::HeapAlloc { .. } => "heap_alloc",
        Operation::Memcpy => "memcpy",
        Operation::Memset => "memset",
        Operation::Global { .. } => "global",
    }
}

// ── Fixture dumps (complex cases with function pointers, structs, globals) ───

#[test]
fn dump_callback_fn_ptr() {
    let bundle = load_ll_bundle("callback_fn_ptr");
    print_air_summary("callback_fn_ptr.ll", &bundle);
}

#[test]
fn dump_null_deref() {
    let bundle = load_ll_bundle("null_deref");
    print_air_summary("null_deref.ll", &bundle);
}

#[test]
fn dump_cg_fptr_struct() {
    let bundle = load_ll_bundle("cg_fptr_struct");
    print_air_summary("cg_fptr_struct.ll", &bundle);
}

#[test]
fn dump_cg_fptr_callback() {
    let bundle = load_ll_bundle("cg_fptr_callback");
    print_air_summary("cg_fptr_callback.ll", &bundle);
}

#[test]
fn dump_vtable_dispatch() {
    let bundle = load_ll_bundle("vtable_dispatch");
    print_air_summary("vtable_dispatch.ll", &bundle);
}

#[test]
fn dump_use_after_free() {
    let bundle = load_ll_bundle("use_after_free");
    print_air_summary("use_after_free.ll", &bundle);
}
