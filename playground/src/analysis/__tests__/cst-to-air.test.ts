import { describe, it, expect, beforeEach } from 'vitest';
import {
  allocaTypeSize,
  typeSize,
  fcmpPredToAir,
  icmpPredToAir,
  extractGepIndices,
  splitTopLevelCommas,
  resolveTextOperand,
  resolvePartOperand,
  extractIndirectCallee,
  extractBalancedParens,
  extractPhiPairs,
  resetIds,
  ValueContext,
} from '@saf/web-shared/analysis';

beforeEach(() => {
  resetIds();
});

// ---------------------------------------------------------------------------
// B2: Alloca size_bytes uses type, not alignment
// ---------------------------------------------------------------------------

describe('allocaTypeSize (B2)', () => {
  it('parses primitive types', () => {
    expect(allocaTypeSize('alloca i32, align 4')).toBe(4);
    expect(allocaTypeSize('alloca i64, align 8')).toBe(8);
    expect(allocaTypeSize('alloca ptr, align 8')).toBe(8);
    expect(allocaTypeSize('alloca i8')).toBe(1);
    expect(allocaTypeSize('alloca i16, align 2')).toBe(2);
    expect(allocaTypeSize('alloca i128')).toBe(16);
    expect(allocaTypeSize('alloca float, align 4')).toBe(4);
    expect(allocaTypeSize('alloca double, align 8')).toBe(8);
    expect(allocaTypeSize('alloca i1')).toBe(1);
  });

  it('parses array types', () => {
    expect(allocaTypeSize('alloca [10 x i32], align 16')).toBe(40);
    expect(allocaTypeSize('alloca [4 x ptr], align 8')).toBe(32);
    expect(allocaTypeSize('alloca [100 x i8]')).toBe(100);
  });

  it('returns undefined for struct/unknown types', () => {
    expect(allocaTypeSize('alloca %struct.Node, align 8')).toBeUndefined();
    expect(allocaTypeSize('alloca %class.Foo')).toBeUndefined();
  });

  it('handles inalloca keyword', () => {
    expect(allocaTypeSize('alloca inalloca i32, align 4')).toBe(4);
  });
});

describe('typeSize', () => {
  it('handles nested array types', () => {
    expect(typeSize('[2 x [3 x i32]]')).toBe(24);
  });

  it('returns undefined for unknown', () => {
    expect(typeSize('%struct.S')).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// B4: FCmp predicate mapping completeness
// ---------------------------------------------------------------------------

describe('fcmpPredToAir (B4)', () => {
  it('maps ordered predicates', () => {
    expect(fcmpPredToAir('oeq')).toBe('f_cmp_oeq');
    expect(fcmpPredToAir('one')).toBe('f_cmp_one');
    expect(fcmpPredToAir('ogt')).toBe('f_cmp_ogt');
    expect(fcmpPredToAir('oge')).toBe('f_cmp_oge');
    expect(fcmpPredToAir('olt')).toBe('f_cmp_olt');
    expect(fcmpPredToAir('ole')).toBe('f_cmp_ole');
  });

  it('maps unordered predicates to nearest ordered equivalent', () => {
    expect(fcmpPredToAir('ueq')).toBe('f_cmp_oeq');
    expect(fcmpPredToAir('une')).toBe('f_cmp_one');
    expect(fcmpPredToAir('ugt')).toBe('f_cmp_ogt');
    expect(fcmpPredToAir('uge')).toBe('f_cmp_oge');
    expect(fcmpPredToAir('ult')).toBe('f_cmp_olt');
    expect(fcmpPredToAir('ule')).toBe('f_cmp_ole');
  });

  it('maps ordering/boolean predicates', () => {
    expect(fcmpPredToAir('ord')).toBe('f_cmp_oeq');
    expect(fcmpPredToAir('uno')).toBe('f_cmp_one');
    expect(fcmpPredToAir('true')).toBe('f_cmp_oeq');
    expect(fcmpPredToAir('false')).toBe('f_cmp_one');
  });
});

describe('icmpPredToAir', () => {
  it('maps all predicates', () => {
    expect(icmpPredToAir('eq')).toBe('i_cmp_eq');
    expect(icmpPredToAir('ne')).toBe('i_cmp_ne');
    expect(icmpPredToAir('slt')).toBe('i_cmp_slt');
    expect(icmpPredToAir('sle')).toBe('i_cmp_sle');
    expect(icmpPredToAir('sgt')).toBe('i_cmp_sgt');
    expect(icmpPredToAir('sge')).toBe('i_cmp_sge');
    expect(icmpPredToAir('ult')).toBe('i_cmp_ult');
    expect(icmpPredToAir('ule')).toBe('i_cmp_ule');
    expect(icmpPredToAir('ugt')).toBe('i_cmp_ugt');
    expect(icmpPredToAir('uge')).toBe('i_cmp_uge');
  });
});

// ---------------------------------------------------------------------------
// B10: GEP comma-split handles aggregate types
// ---------------------------------------------------------------------------

describe('extractGepIndices (B10)', () => {
  it('extracts numeric indices from simple GEP', () => {
    const text = 'getelementptr inbounds %struct.S, ptr %s, i32 0, i32 1';
    const indices = extractGepIndices(text);
    expect(indices).toEqual([0, 1]);
  });

  it('extracts variable index', () => {
    const text = 'getelementptr inbounds i32, ptr %arr, i64 %idx';
    const indices = extractGepIndices(text);
    expect(indices).toEqual(['%idx']);
  });

  it('handles aggregate types without breaking on inner commas', () => {
    const text = 'getelementptr inbounds { i32, ptr }, ptr %s, i32 0, i32 1';
    const indices = extractGepIndices(text);
    expect(indices).toEqual([0, 1]);
  });

  it('handles nested aggregate types', () => {
    const text = 'getelementptr inbounds { i32, { ptr, i64 } }, ptr %s, i32 0, i32 1, i32 0';
    const indices = extractGepIndices(text);
    expect(indices).toEqual([0, 1, 0]);
  });
});

describe('splitTopLevelCommas', () => {
  it('splits simple comma-separated values', () => {
    const parts = splitTopLevelCommas('a, b, c');
    expect(parts).toEqual(['a', ' b', ' c']);
  });

  it('does not split inside braces', () => {
    const parts = splitTopLevelCommas('{ i32, ptr }, ptr %s, i32 0');
    expect(parts).toEqual(['{ i32, ptr }', ' ptr %s', ' i32 0']);
  });

  it('does not split inside brackets', () => {
    const parts = splitTopLevelCommas('[2 x i32], ptr %arr, i32 0');
    expect(parts).toEqual(['[2 x i32]', ' ptr %arr', ' i32 0']);
  });
});

// ---------------------------------------------------------------------------
// B11: Literal operand resolution
// ---------------------------------------------------------------------------

describe('resolveTextOperand (B11)', () => {
  it('resolves local variables', () => {
    const globals = new Map<string, string>();
    const funcIds = new Map<string, string>();
    const ctx = new ValueContext(globals, funcIds);
    const id = resolveTextOperand('%x', ctx);
    expect(id).toMatch(/^0x/);

    // Same variable should resolve to same ID
    const id2 = resolveTextOperand('%x', ctx);
    expect(id2).toBe(id);
  });

  it('resolves global variables', () => {
    const globals = new Map<string, string>([['g', '0x00000000000000000000000000000099']]);
    const funcIds = new Map<string, string>();
    const ctx = new ValueContext(globals, funcIds);
    expect(resolveTextOperand('@g', ctx)).toBe('0x00000000000000000000000000000099');
  });

  it('resolves null literal', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolveTextOperand('null', ctx);
    expect(id).toMatch(/^0x/);
  });

  it('resolves undef literal', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolveTextOperand('undef', ctx);
    expect(id).toMatch(/^0x/);
  });

  it('resolves true/false literals', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const trueId = resolveTextOperand('true', ctx);
    const falseId = resolveTextOperand('false', ctx);
    expect(trueId).toMatch(/^0x/);
    expect(falseId).toMatch(/^0x/);
    // Each literal gets a fresh (different) ID
    expect(trueId).not.toBe(falseId);
  });

  it('resolves zeroinitializer literal', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolveTextOperand('zeroinitializer', ctx);
    expect(id).toMatch(/^0x/);
  });

  it('resolves numeric constants', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolveTextOperand('42', ctx);
    expect(id).toMatch(/^0x/);
  });
});

// ---------------------------------------------------------------------------
// Positional operand resolution (fixes audit bugs #1-#9)
// ---------------------------------------------------------------------------

describe('resolvePartOperand', () => {
  it('resolves SSA local variables with type prefix', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolvePartOperand('i32 %x', ctx);
    expect(id).toMatch(/^0x/);
    // Same variable resolves to same ID
    expect(resolvePartOperand('ptr %x', ctx)).toBe(id);
  });

  it('resolves SSA global variables', () => {
    const globals = new Map([['g', '0x00000000000000000000000000000099']]);
    const ctx = new ValueContext(globals, new Map());
    expect(resolvePartOperand('ptr @g', ctx)).toBe('0x00000000000000000000000000000099');
  });

  it('resolves numeric constants (Bug #2: icmp/binop with constants)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    // "i32 0" from "icmp eq i32 %x, 0" second operand
    const id = resolvePartOperand('i32 0', ctx);
    expect(id).toMatch(/^0x/);
    // Negative numbers
    expect(resolvePartOperand('i32 -1', ctx)).toMatch(/^0x/);
    // Large numbers
    expect(resolvePartOperand('i64 42', ctx)).toMatch(/^0x/);
  });

  it('resolves null literals (Bug #1/#5: null position)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolvePartOperand('ptr null', ctx);
    expect(id).toMatch(/^0x/);
  });

  it('resolves undef/poison/zeroinitializer', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('ptr undef', ctx)).toMatch(/^0x/);
    expect(resolvePartOperand('i32 poison', ctx)).toMatch(/^0x/);
    expect(resolvePartOperand('ptr zeroinitializer', ctx)).toMatch(/^0x/);
  });

  it('resolves boolean literals (Bug #7: br with true/false)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('i1 true', ctx)).toMatch(/^0x/);
    expect(resolvePartOperand('i1 false', ctx)).toMatch(/^0x/);
  });

  it('resolves float constants', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('float 1.5', ctx)).toMatch(/^0x/);
    expect(resolvePartOperand('double 1.5e+2', ctx)).toMatch(/^0x/);
    expect(resolvePartOperand('double -0.0', ctx)).toMatch(/^0x/);
  });

  it('returns null for bare types (not operands)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('i32', ctx)).toBeNull();
    expect(resolvePartOperand('ptr', ctx)).toBeNull();
    expect(resolvePartOperand('void', ctx)).toBeNull();
    expect(resolvePartOperand('float', ctx)).toBeNull();
    expect(resolvePartOperand('double', ctx)).toBeNull();
  });

  it('returns null for label operands', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('label %then', ctx)).toBeNull();
  });

  it('returns null for type references', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('%struct.Node', ctx)).toBeNull();
    expect(resolvePartOperand('%class.Foo', ctx)).toBeNull();
  });

  it('strips align attribute (not a value)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    // "align 4" is not a value operand
    expect(resolvePartOperand('align 4', ctx)).toBeNull();
  });

  it('handles cast destination type stripping (Bug #9: inttoptr)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    // "i64 0 to ptr" should resolve the 0, not "ptr"
    const id = resolvePartOperand('i64 0 to ptr', ctx);
    expect(id).toMatch(/^0x/);
    // SSA in cast
    const id2 = resolvePartOperand('i64 %x to ptr', ctx);
    expect(id2).toMatch(/^0x/);
  });

  it('handles noundef/nonnull attributes', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = resolvePartOperand('ptr noundef %p', ctx);
    expect(id).toMatch(/^0x/);
    // Same %p should resolve to same ID
    expect(resolvePartOperand('ptr %p', ctx)).toBe(id);
  });

  it('returns null for empty string', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(resolvePartOperand('', ctx)).toBeNull();
  });

  it('resolves values with named struct type prefix (Bug #3)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    // "%struct.S %x" should resolve %x, not be filtered as a type reference
    const id = resolvePartOperand('%struct.S %x', ctx);
    expect(id).toMatch(/^0x/);
    // Verify it resolved %x specifically
    expect(resolvePartOperand('ptr %x', ctx)).toBe(id);
  });

  it('still filters bare struct type references (Bug #3)', () => {
    const ctx = new ValueContext(new Map(), new Map());
    // Bare type references without a trailing value should still be null
    expect(resolvePartOperand('%struct.Node', ctx)).toBeNull();
    expect(resolvePartOperand('%class.Foo', ctx)).toBeNull();
    expect(resolvePartOperand('%union.Data', ctx)).toBeNull();
    expect(resolvePartOperand('%struct.S*', ctx)).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Bug #1+2+6: Indirect call function pointer extraction (callee-LAST)
// ---------------------------------------------------------------------------

describe('extractIndirectCallee (Bug #1+2+6)', () => {
  it('extracts local register callee from indirect call', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = extractIndirectCallee('call i32 %0(i32 %1, i32 %2)', ctx);
    expect(id).toMatch(/^0x/);
    // Should resolve to the same ID as %0
    expect(ctx.resolveLocal('0')).toBe(id);
  });

  it('extracts named local callee', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = extractIndirectCallee('call void %fptr(ptr %arg)', ctx);
    expect(id).toMatch(/^0x/);
    expect(ctx.resolveLocal('fptr')).toBe(id);
  });

  it('extracts global callee from indirect call', () => {
    const funcIds = new Map([['thunk', '0x00000000000000000000000000000042']]);
    const ctx = new ValueContext(new Map(), funcIds);
    const id = extractIndirectCallee('call void @thunk(ptr %p)', ctx);
    expect(id).toBe('0x00000000000000000000000000000042');
  });

  it('returns undefined when no callee is found', () => {
    const ctx = new ValueContext(new Map(), new Map());
    expect(extractIndirectCallee('ret void', ctx)).toBeUndefined();
  });

  it('handles full instruction with destination', () => {
    const ctx = new ValueContext(new Map(), new Map());
    const id = extractIndirectCallee('%4 = call i32 %0(i32 %1, i32 %2)', ctx);
    expect(id).toMatch(/^0x/);
    expect(ctx.resolveLocal('0')).toBe(id);
  });
});

// ---------------------------------------------------------------------------
// Bug #5: Balanced parenthesis extraction
// ---------------------------------------------------------------------------

describe('extractBalancedParens (Bug #5)', () => {
  it('extracts simple parenthesized content', () => {
    expect(extractBalancedParens('foo(a, b, c)', 3)).toBe('a, b, c');
  });

  it('handles nested parentheses', () => {
    expect(extractBalancedParens('foo(bitcast(ptr @bar to ptr), i32 %x)', 3))
      .toBe('bitcast(ptr @bar to ptr), i32 %x');
  });

  it('handles empty parentheses', () => {
    expect(extractBalancedParens('foo()', 3)).toBe('');
  });

  it('returns null for non-paren start', () => {
    expect(extractBalancedParens('foo bar', 3)).toBeNull();
  });

  it('returns null for unbalanced parentheses', () => {
    expect(extractBalancedParens('foo(bar', 3)).toBeNull();
  });

  it('handles deeply nested parens', () => {
    expect(extractBalancedParens('f(a(b(c)), d)', 1))
      .toBe('a(b(c)), d');
  });
});

// ---------------------------------------------------------------------------
// Bug #4: PHI aggregate value parsing
// ---------------------------------------------------------------------------

describe('extractPhiPairs (Bug #4)', () => {
  it('extracts simple phi pairs', () => {
    const pairs = extractPhiPairs('phi i32 [ %v1, %bb1 ], [ %v2, %bb2 ]');
    expect(pairs).toEqual([
      ['%v1', 'bb1'],
      ['%v2', 'bb2'],
    ]);
  });

  it('extracts phi pairs with numeric constants', () => {
    const pairs = extractPhiPairs('phi i32 [ 0, %entry ], [ %inc, %loop ]');
    expect(pairs).toEqual([
      ['0', 'entry'],
      ['%inc', 'loop'],
    ]);
  });

  it('handles aggregate values with internal commas', () => {
    const pairs = extractPhiPairs(
      'phi { i32, ptr } [ { i32 0, ptr null }, %bb1 ], [ { i32 1, ptr @g }, %bb2 ]',
    );
    expect(pairs).toEqual([
      ['{ i32 0, ptr null }', 'bb1'],
      ['{ i32 1, ptr @g }', 'bb2'],
    ]);
  });

  it('handles null values', () => {
    const pairs = extractPhiPairs('phi ptr [ null, %entry ], [ %p, %loop ]');
    expect(pairs).toEqual([
      ['null', 'entry'],
      ['%p', 'loop'],
    ]);
  });

  it('returns empty array for non-phi text', () => {
    expect(extractPhiPairs('add i32 %x, %y')).toEqual([]);
  });
});
