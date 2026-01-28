//! Debug metadata extraction from LLVM IR.
//!
//! Best-effort extraction of source locations (`DILocation`) and symbol
//! information (`DISubprogram`) from LLVM debug metadata. Never fails
//! ingestion due to missing or malformed debug info.
//!
//! Uses inkwell 0.8's `get_debug_location()` for instruction-level spans
//! and `llvm-sys` FFI for file info (`LLVMDIScopeGetFile`,
//! `LLVMDIFileGetFilename`, `LLVMDIFileGetDirectory`).

use std::collections::BTreeMap;

use saf_core::ids::FileId;
use saf_core::span::SourceFile;
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
use saf_core::span::{Span, Symbol};

#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
use inkwell::debug_info::AsDIScope;
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
use inkwell::values::{FunctionValue, InstructionValue};

/// Tracker for source files encountered during ingestion.
///
/// Maintains a mapping from (filename, directory) to file ID for
/// efficient Span construction.
#[derive(Debug, Default)]
pub struct SourceFileTracker {
    #[allow(dead_code)] // Used only when LLVM feature is active
    files: BTreeMap<(String, String), FileId>,
    file_list: Vec<SourceFile>,
}

impl SourceFileTracker {
    /// Create a new source file tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a `FileId` for the given file/directory pair.
    #[allow(dead_code)] // Used only when LLVM feature is active
    pub fn get_or_create(&mut self, filename: &str, directory: &str) -> FileId {
        let key = (filename.to_string(), directory.to_string());
        if let Some(&id) = self.files.get(&key) {
            return id;
        }

        let path = if directory.is_empty() {
            filename.to_string()
        } else {
            format!("{directory}/{filename}")
        };

        let id = FileId::derive(path.as_bytes());
        self.file_list.push(SourceFile::new(id, path));
        self.files.insert(key, id);
        id
    }

    /// Consume the tracker and return the list of source files.
    #[must_use]
    pub fn into_files(self) -> Vec<SourceFile> {
        self.file_list
    }
}

/// Extract a `Span` from an LLVM instruction's debug location.
///
/// Uses inkwell's `get_debug_location()` for line/column and `llvm-sys` FFI
/// for file info. Returns `None` when debug metadata is absent.
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
pub fn extract_span(inst: InstructionValue<'_>, files: &mut SourceFileTracker) -> Option<Span> {
    let di_loc = inst.get_debug_location()?;
    let line = di_loc.get_line();
    let col = di_loc.get_column();

    let scope = di_loc.get_scope();
    // SAFETY: scope.as_mut_ptr() is a valid LLVMMetadataRef from inkwell
    let (filename, directory) = unsafe { get_file_from_scope(scope.as_mut_ptr()) }?;

    let file_id = files.get_or_create(&filename, &directory);
    Some(Span::point(file_id, 0, line, col))
}

/// Extract file path from an LLVM `DIScope` via `llvm-sys` FFI.
///
/// # Safety
///
/// `scope_ref` must be a valid `LLVMMetadataRef` pointing to a `DIScope`.
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
unsafe fn get_file_from_scope(
    scope_ref: inkwell::llvm_sys::prelude::LLVMMetadataRef,
) -> Option<(String, String)> {
    // SAFETY: all calls below require scope_ref/file_ref to be valid LLVM metadata
    // references, which is guaranteed by the caller.
    let file_ref = unsafe { inkwell::llvm_sys::debuginfo::LLVMDIScopeGetFile(scope_ref) };
    if file_ref.is_null() {
        return None;
    }

    let mut filename_len: std::ffi::c_uint = 0;
    let filename_ptr =
        unsafe { inkwell::llvm_sys::debuginfo::LLVMDIFileGetFilename(file_ref, &mut filename_len) };
    if filename_ptr.is_null() {
        return None;
    }
    let filename = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(
            filename_ptr.cast::<u8>(),
            filename_len as usize,
        ))
    }
    .ok()?
    .to_string();

    let mut dir_len: std::ffi::c_uint = 0;
    let dir_ptr =
        unsafe { inkwell::llvm_sys::debuginfo::LLVMDIFileGetDirectory(file_ref, &mut dir_len) };
    let directory = if dir_ptr.is_null() {
        String::new()
    } else {
        unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(
                dir_ptr.cast::<u8>(),
                dir_len as usize,
            ))
        }
        .ok()?
        .to_string()
    };

    Some((filename, directory))
}

/// Extract a `Symbol` from a function's debug information.
///
/// Uses `DISubprogram` metadata when available. Falls back to the LLVM
/// function name, which is always present. For C code, the function name
/// is the display name. For C++, it is the mangled linkage name.
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
pub fn extract_function_symbol(func: FunctionValue<'_>) -> Option<Symbol> {
    let llvm_name = func.get_name().to_str().ok()?;

    // Skip LLVM intrinsics — they are not user-visible symbols
    if llvm_name.starts_with("llvm.") {
        return None;
    }

    // If a subprogram exists, the LLVM function name is the linkage/mangled name
    // and we store it as such. Without higher-level demangling support, the
    // display_name defaults to the same string.
    let has_debug = func.get_subprogram().is_some();

    Some(Symbol {
        display_name: llvm_name.to_string(),
        mangled_name: if has_debug {
            Some(llvm_name.to_string())
        } else {
            None
        },
        namespace_path: Vec::new(),
    })
}

/// Extract a `Span` for a function from its `DISubprogram`.
///
/// Uses `get_subprogram()` for the function declaration line and `llvm-sys`
/// FFI for file info. Returns `None` when debug metadata is absent.
#[cfg(any(feature = "llvm-17", feature = "llvm-18"))]
pub fn extract_function_span(
    func: FunctionValue<'_>,
    files: &mut SourceFileTracker,
) -> Option<Span> {
    let subprogram = func.get_subprogram()?;

    // Get line number via llvm-sys (DISubprogram doesn't expose get_line in inkwell)
    let line =
        unsafe { inkwell::llvm_sys::debuginfo::LLVMDISubprogramGetLine(subprogram.as_mut_ptr()) };

    // Get file from the subprogram's scope
    let scope_ref = subprogram.as_debug_info_scope().as_mut_ptr();
    // SAFETY: scope_ref is a valid LLVMMetadataRef from inkwell
    let (filename, directory) = unsafe { get_file_from_scope(scope_ref) }?;

    let file_id = files.get_or_create(&filename, &directory);
    Some(Span::point(file_id, 0, line, 0))
}

/// Parsed local variable name mappings from LLVM debug info.
///
/// Maps function names to their local variable debug info. For each function,
/// maps LLVM register names (e.g., `%2`, `%p.addr`) to C/C++ variable names
/// extracted from `DILocalVariable` metadata via `llvm.dbg.declare` intrinsics.
pub type LocalVarNameMap = BTreeMap<String, BTreeMap<String, String>>;

/// Extract local variable names from LLVM IR module text.
///
/// Parses `llvm.dbg.declare` / `#dbg_declare` intrinsics and `DILocalVariable`
/// metadata to build a mapping from (`function_name`, `register_name`) to the
/// source-level variable name. Supports both the old-style intrinsic call
/// format and LLVM 18's new debug record format.
///
/// This function is best-effort: missing or malformed debug info is silently
/// skipped. The returned map may be empty if no debug info is present.
///
/// # Arguments
///
/// * `module_ir` - The full LLVM IR text of the module (from `module.print_to_string()`)
#[must_use]
pub fn extract_local_variable_names(module_ir: &str) -> LocalVarNameMap {
    // Phase 1: Parse all DILocalVariable metadata nodes.
    // Format: !18 = !DILocalVariable(name: "p", scope: !13, ...)
    let mut metadata_names: BTreeMap<String, String> = BTreeMap::new();
    for line in module_ir.lines() {
        let trimmed = line.trim();
        if let Some((meta_id, var_name)) = parse_di_local_variable(trimmed) {
            metadata_names.insert(meta_id, var_name);
        }
    }

    if metadata_names.is_empty() {
        return BTreeMap::new();
    }

    // Phase 2: Parse dbg.declare intrinsics within function contexts.
    // Track current function by detecting `define` lines.
    let mut result: LocalVarNameMap = BTreeMap::new();
    let mut current_func: Option<String> = None;

    for line in module_ir.lines() {
        let trimmed = line.trim();

        // Detect function boundaries
        if let Some(func_name) = parse_function_define(trimmed) {
            current_func = Some(func_name);
            continue;
        }
        if trimmed == "}" {
            current_func = None;
            continue;
        }

        // Only parse dbg.declare inside functions
        let Some(ref func_name) = current_func else {
            continue;
        };

        // Try to parse dbg.declare (old-style or new-style)
        if let Some((reg_name, meta_id)) = parse_dbg_declare(trimmed) {
            if let Some(var_name) = metadata_names.get(&meta_id) {
                result
                    .entry(func_name.clone())
                    .or_default()
                    .insert(reg_name, var_name.clone());
            }
        }
    }

    result
}

/// Parse a `DILocalVariable` metadata line.
///
/// Returns `(metadata_id, variable_name)` if the line matches.
/// Example: `!18 = !DILocalVariable(name: "p", scope: !13, ...)` -> `("!18", "p")`
fn parse_di_local_variable(line: &str) -> Option<(String, String)> {
    // Match: !N = !DILocalVariable(...)
    if !line.contains("!DILocalVariable(") {
        return None;
    }

    // Extract metadata ID: everything before " = "
    let eq_pos = line.find(" = ")?;
    let meta_id = line[..eq_pos].trim().to_string();
    if !meta_id.starts_with('!') {
        return None;
    }

    // Extract name field: name: "..."
    let name_marker = "name: \"";
    let name_start = line.find(name_marker)? + name_marker.len();
    let name_end = line[name_start..].find('"')? + name_start;
    let var_name = line[name_start..name_end].to_string();

    Some((meta_id, var_name))
}

/// Parse a `define` line to extract the function name.
///
/// Returns the function name if the line is a function definition.
/// Example: `define dso_local i32 @main() #0 !dbg !13 {` -> `"main"`
fn parse_function_define(line: &str) -> Option<String> {
    if !line.starts_with("define ") {
        return None;
    }

    // Find @name pattern
    let at_pos = line.find('@')?;
    let name_start = at_pos + 1;
    let rest = &line[name_start..];
    // Function name ends at '(' or space
    let name_end = rest.find(['(', ' '])?;
    Some(rest[..name_end].to_string())
}

/// Parse a `dbg.declare` intrinsic (old-style or new-style) to extract
/// the register name and metadata reference.
///
/// Old-style: `call void @llvm.dbg.declare(metadata ptr %2, metadata !18, ...)`
/// New-style: `#dbg_declare(ptr %2, !18, !DIExpression(), !19)`
///
/// Returns `(register_name, metadata_id)` on success, e.g. `("%2", "!18")`.
fn parse_dbg_declare(line: &str) -> Option<(String, String)> {
    if line.contains("@llvm.dbg.declare(") {
        parse_old_style_dbg_declare(line)
    } else if line.contains("#dbg_declare(") {
        parse_new_style_dbg_declare(line)
    } else {
        None
    }
}

/// Parse old-style `llvm.dbg.declare` intrinsic call.
///
/// Format: `call void @llvm.dbg.declare(metadata ptr %reg, metadata !N, ...)`
/// Also handles: `call void @llvm.dbg.declare(metadata %reg, metadata !N, ...)`
fn parse_old_style_dbg_declare(line: &str) -> Option<(String, String)> {
    let marker = "@llvm.dbg.declare(";
    let start = line.find(marker)? + marker.len();
    let args = &line[start..];

    // First arg: "metadata ptr %reg" or "metadata %reg"
    // Extract register name: find '%' then take until ',' or ')'
    let pct_pos = args.find('%')?;
    let reg_rest = &args[pct_pos..];
    let reg_end = reg_rest.find([',', ')'])?;
    let reg_name = reg_rest[..reg_end].trim().to_string();

    // Second arg: "metadata !N"
    // Find the second "metadata" after the comma
    let comma_pos = args.find(',')?;
    let after_comma = &args[comma_pos + 1..];
    // Find !N (metadata reference)
    let meta_start = after_comma.find('!')?;
    let meta_rest = &after_comma[meta_start..];
    // Metadata ID ends at ',' or ')' or space followed by non-digit
    let meta_end = meta_rest[1..]
        .find(|c: char| !c.is_ascii_digit())
        .map_or(meta_rest.len(), |pos| pos + 1);
    let meta_id = meta_rest[..meta_end].to_string();

    if meta_id.len() > 1 && reg_name.starts_with('%') {
        Some((reg_name, meta_id))
    } else {
        None
    }
}

/// Parse LLVM 18 new-style `#dbg_declare` record.
///
/// Format: `#dbg_declare(ptr %reg, !N, !DIExpression(), !M)`
fn parse_new_style_dbg_declare(line: &str) -> Option<(String, String)> {
    let marker = "#dbg_declare(";
    let start = line.find(marker)? + marker.len();
    let args = &line[start..];

    // First arg: "ptr %reg" or just "%reg"
    let pct_pos = args.find('%')?;
    let reg_rest = &args[pct_pos..];
    let reg_end = reg_rest.find([',', ')'])?;
    let reg_name = reg_rest[..reg_end].trim().to_string();

    // Second arg: !N
    let comma_pos = args.find(',')?;
    let after_comma = &args[comma_pos + 1..];
    let meta_start = after_comma.find('!')?;
    let meta_rest = &after_comma[meta_start..];
    let meta_end = meta_rest[1..]
        .find(|c: char| !c.is_ascii_digit())
        .map_or(meta_rest.len(), |pos| pos + 1);
    let meta_id = meta_rest[..meta_end].to_string();

    if meta_id.len() > 1 && reg_name.starts_with('%') {
        Some((reg_name, meta_id))
    } else {
        None
    }
}

/// Extract the LLVM register name from an instruction's text representation.
///
/// For instructions with results (e.g., `%2 = alloca ptr, align 8`),
/// returns the register name (e.g., `%2`). Returns `None` for instructions
/// without results (e.g., `store`, `ret`).
pub fn extract_register_name(inst_str: &str) -> Option<String> {
    let trimmed = inst_str.trim();
    // Instructions with results have format: %name = opcode ...
    if !trimmed.starts_with('%') {
        return None;
    }
    let eq_pos = trimmed.find(" = ")?;
    Some(trimmed[..eq_pos].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_file_tracker_deduplicates() {
        let mut tracker = SourceFileTracker::new();
        let id1 = tracker.get_or_create("foo.c", "/src");
        let id2 = tracker.get_or_create("bar.c", "/src");
        let id3 = tracker.get_or_create("foo.c", "/src"); // duplicate

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);

        let files = tracker.into_files();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn source_file_tracker_handles_empty_directory() {
        let mut tracker = SourceFileTracker::new();
        tracker.get_or_create("test.c", "");

        let files = tracker.into_files();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "test.c");
    }

    #[test]
    fn source_file_tracker_builds_path() {
        let mut tracker = SourceFileTracker::new();
        tracker.get_or_create("main.c", "/home/user/project");

        let files = tracker.into_files();
        assert_eq!(files[0].path, "/home/user/project/main.c");
    }

    #[test]
    fn parse_di_local_variable_simple() {
        let line = r#"!18 = !DILocalVariable(name: "p", scope: !13, file: !14, line: 8, type: !3)"#;
        let (meta_id, var_name) = parse_di_local_variable(line).unwrap();
        assert_eq!(meta_id, "!18");
        assert_eq!(var_name, "p");
    }

    #[test]
    fn parse_di_local_variable_longer_name() {
        let line = r#"!42 = !DILocalVariable(name: "my_variable", scope: !5, file: !6, line: 3, type: !7)"#;
        let (meta_id, var_name) = parse_di_local_variable(line).unwrap();
        assert_eq!(meta_id, "!42");
        assert_eq!(var_name, "my_variable");
    }

    #[test]
    fn parse_di_local_variable_not_matching() {
        assert!(parse_di_local_variable("!0 = !DICompileUnit(language: DW_LANG_C11)").is_none());
        assert!(parse_di_local_variable("define dso_local i32 @main()").is_none());
    }

    #[test]
    fn parse_function_define_simple() {
        let line = "define dso_local i32 @main() #0 !dbg !13 {";
        assert_eq!(parse_function_define(line), Some("main".to_string()));
    }

    #[test]
    fn parse_function_define_with_args() {
        let line = "define dso_local void @foo(i32 %x, ptr %p) #0 !dbg !5 {";
        assert_eq!(parse_function_define(line), Some("foo".to_string()));
    }

    #[test]
    fn parse_function_define_not_matching() {
        assert!(parse_function_define("declare void @free(ptr)").is_none());
        assert!(parse_function_define("  %1 = alloca i32").is_none());
    }

    #[test]
    fn parse_old_style_dbg_declare_basic() {
        let line = "  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19";
        let (reg, meta) = parse_dbg_declare(line).unwrap();
        assert_eq!(reg, "%2");
        assert_eq!(meta, "!18");
    }

    #[test]
    fn parse_old_style_dbg_declare_named_reg() {
        let line = "  call void @llvm.dbg.declare(metadata ptr %p.addr, metadata !22, metadata !DIExpression()), !dbg !23";
        let (reg, meta) = parse_dbg_declare(line).unwrap();
        assert_eq!(reg, "%p.addr");
        assert_eq!(meta, "!22");
    }

    #[test]
    fn parse_new_style_dbg_declare_basic() {
        let line = "    #dbg_declare(ptr %2, !18, !DIExpression(), !19)";
        let (reg, meta) = parse_dbg_declare(line).unwrap();
        assert_eq!(reg, "%2");
        assert_eq!(meta, "!18");
    }

    #[test]
    fn extract_register_name_alloca() {
        assert_eq!(
            extract_register_name("  %2 = alloca ptr, align 8"),
            Some("%2".to_string())
        );
    }

    #[test]
    fn extract_register_name_named() {
        assert_eq!(
            extract_register_name("  %p.addr = alloca ptr, align 8"),
            Some("%p.addr".to_string())
        );
    }

    #[test]
    fn extract_register_name_no_result() {
        assert!(extract_register_name("  store i32 0, ptr %1, align 4").is_none());
        assert!(extract_register_name("  ret i32 0").is_none());
    }

    #[test]
    fn extract_local_variable_names_full() {
        let module_ir = r#"
define dso_local i32 @main() #0 !dbg !13 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  store i32 0, ptr %1, align 4
  call void @llvm.dbg.declare(metadata ptr %2, metadata !18, metadata !DIExpression()), !dbg !19
  ret i32 0
}

!18 = !DILocalVariable(name: "p", scope: !13, file: !14, line: 8, type: !3)
"#;
        let map = extract_local_variable_names(module_ir);
        let main_vars = map.get("main").unwrap();
        assert_eq!(main_vars.get("%2"), Some(&"p".to_string()));
        assert!(main_vars.get("%1").is_none());
    }

    #[test]
    fn extract_local_variable_names_multiple_vars() {
        let module_ir = r#"
define dso_local void @foo(i32 %x) #0 !dbg !5 {
  %1 = alloca i32, align 4
  %2 = alloca ptr, align 8
  %3 = alloca i32, align 4
  call void @llvm.dbg.declare(metadata ptr %1, metadata !10, metadata !DIExpression()), !dbg !11
  call void @llvm.dbg.declare(metadata ptr %2, metadata !12, metadata !DIExpression()), !dbg !13
  call void @llvm.dbg.declare(metadata ptr %3, metadata !14, metadata !DIExpression()), !dbg !15
  ret void
}

!10 = !DILocalVariable(name: "x", scope: !5, file: !6, line: 1, type: !7)
!12 = !DILocalVariable(name: "ptr", scope: !5, file: !6, line: 2, type: !8)
!14 = !DILocalVariable(name: "count", scope: !5, file: !6, line: 3, type: !7)
"#;
        let map = extract_local_variable_names(module_ir);
        let foo_vars = map.get("foo").unwrap();
        assert_eq!(foo_vars.get("%1"), Some(&"x".to_string()));
        assert_eq!(foo_vars.get("%2"), Some(&"ptr".to_string()));
        assert_eq!(foo_vars.get("%3"), Some(&"count".to_string()));
    }

    #[test]
    fn extract_local_variable_names_no_debug_info() {
        let module_ir = r"
define dso_local i32 @main() {
  %1 = alloca i32, align 4
  ret i32 0
}
";
        let map = extract_local_variable_names(module_ir);
        assert!(map.is_empty());
    }
}
