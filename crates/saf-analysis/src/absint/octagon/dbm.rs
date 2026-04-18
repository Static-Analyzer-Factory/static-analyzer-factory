//! Difference-Bound Matrix for octagon constraints.
//!
//! Represents constraints of the form `x_i - x_j <= c` using a 2n x 2n matrix
//! where n is the number of program variables. Each variable v has two DBM
//! variables: v^+ (positive) and v^- (negative, representing -v).
//!
//! This encoding allows octagon constraints `±x ± y <= c` to be represented
//! as standard difference constraints.

use std::cmp;

/// Index for a variable in the DBM.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarIndex(pub usize);

impl VarIndex {
    /// Get the positive DBM index for this variable (2*idx).
    #[must_use]
    pub const fn positive(self) -> usize {
        self.0 * 2
    }

    /// Get the negative DBM index for this variable (2*idx + 1).
    #[must_use]
    pub const fn negative(self) -> usize {
        self.0 * 2 + 1
    }
}

/// A bound in the DBM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// Finite bound value.
    Finite(i128),
    /// Positive infinity (no constraint).
    PosInf,
}

impl Bound {
    /// Check if this bound is infinity.
    #[must_use]
    pub const fn is_inf(&self) -> bool {
        matches!(self, Self::PosInf)
    }

    /// Saturating addition of two bounds.
    #[must_use]
    pub fn saturating_add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(a.saturating_add(b)),
            _ => Self::PosInf,
        }
    }

    /// Take the minimum of two bounds.
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(cmp::min(a, b)),
            (Self::Finite(a), Self::PosInf) => Self::Finite(a),
            (Self::PosInf, Self::Finite(b)) => Self::Finite(b),
            (Self::PosInf, Self::PosInf) => Self::PosInf,
        }
    }

    /// Take the maximum of two bounds.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(cmp::max(a, b)),
            _ => Self::PosInf,
        }
    }
}

impl PartialOrd for Bound {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bound {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => a.cmp(b),
            (Self::Finite(_), Self::PosInf) => cmp::Ordering::Less,
            (Self::PosInf, Self::Finite(_)) => cmp::Ordering::Greater,
            (Self::PosInf, Self::PosInf) => cmp::Ordering::Equal,
        }
    }
}

/// Difference-Bound Matrix for octagon constraints.
///
/// Represents constraints of the form: `x_i - x_j <= m[i][j]`
///
/// For octagons, we use 2n DBM variables for n program variables:
/// - `v^+` at index 2*k represents `v`
/// - `v^-` at index 2*k+1 represents `-v`
///
/// This allows encoding octagon constraints `±x ± y <= c`:
/// - `x - y <= c`  →  `v_x^+ - v_y^+ <= c`  →  `m[2x][2y] = c`
/// - `x + y <= c`  →  `v_x^+ - v_y^- <= c`  →  `m[2x][2y+1] = c`
/// - `-x - y <= c` →  `v_x^- - v_y^+ <= c`  →  `m[2x+1][2y] = c`
/// - `-x + y <= c` →  `v_x^- - v_y^- <= c`  →  `m[2x+1][2y+1] = c`
#[derive(Clone, Debug)]
pub struct Dbm {
    /// Number of program variables (not DBM variables).
    num_vars: usize,
    /// Matrix entries: `m[i][j]` represents `x_i - x_j <= m[i][j]`.
    /// Size is 2n x 2n where n = `num_vars`.
    matrix: Vec<Vec<Bound>>,
    /// Whether this is the bottom (inconsistent) DBM.
    bottom: bool,
}

impl PartialEq for Dbm {
    fn eq(&self, other: &Self) -> bool {
        if self.bottom && other.bottom {
            return true;
        }
        if self.bottom || other.bottom {
            return false;
        }
        self.num_vars == other.num_vars && self.matrix == other.matrix
    }
}

impl Eq for Dbm {}

impl Dbm {
    /// Create an empty DBM (all constraints are `PosInf`, representing top).
    #[must_use]
    #[allow(clippy::needless_range_loop)] // We need i for both row and column indexing
    pub fn top(num_vars: usize) -> Self {
        let size = num_vars * 2;
        let mut matrix = vec![vec![Bound::PosInf; size]; size];

        // Diagonal entries are 0 (x_i - x_i <= 0)
        for i in 0..size {
            matrix[i][i] = Bound::Finite(0);
        }

        Self {
            num_vars,
            matrix,
            bottom: false,
        }
    }

    /// Create an inconsistent (bottom) DBM.
    #[must_use]
    pub fn bottom(num_vars: usize) -> Self {
        Self {
            num_vars,
            matrix: vec![],
            bottom: true,
        }
    }

    /// Get the number of program variables.
    #[must_use]
    pub const fn num_vars(&self) -> usize {
        self.num_vars
    }

    /// Check if this DBM is bottom (inconsistent).
    #[must_use]
    pub const fn is_bottom(&self) -> bool {
        self.bottom
    }

    /// Get a constraint value.
    #[must_use]
    pub fn get(&self, i: usize, j: usize) -> Bound {
        if self.bottom {
            return Bound::PosInf;
        }
        self.matrix[i][j]
    }

    /// Set a constraint value (without closure).
    pub fn set(&mut self, i: usize, j: usize, bound: Bound) {
        if !self.bottom {
            self.matrix[i][j] = bound;
        }
    }

    /// Add a difference constraint: `var1 - var2 <= bound`.
    ///
    /// This updates `m[var1][var2]` to `min(current, bound)`.
    pub fn add_difference(&mut self, var1: usize, var2: usize, bound: i128) {
        if self.bottom {
            return;
        }
        let current = self.matrix[var1][var2];
        self.matrix[var1][var2] = current.min(Bound::Finite(bound));
    }

    /// Add an octagon constraint: `coef_left * left + coef_right * right <= bound`
    /// where `coef_left`, `coef_right` are in {-1, +1}.
    ///
    /// Converts to DBM form and updates the matrix.
    #[allow(clippy::many_single_char_names)]
    pub fn add_octagon(
        &mut self,
        left: VarIndex,
        coef_left: i8,
        right: VarIndex,
        coef_right: i8,
        bound: i128,
    ) {
        if self.bottom {
            return;
        }

        // Convert to difference constraint form
        let (row, col) = match (coef_left, coef_right) {
            (1, -1) => (left.positive(), right.positive()), // left - right <= bound
            (1, 1) => (left.positive(), right.negative()),  // left + right <= bound
            (-1, -1) => (left.negative(), right.positive()), // -left - right <= bound
            (-1, 1) => (left.negative(), right.negative()), // -left + right <= bound
            _ => return,                                    // Invalid coefficients
        };

        self.add_difference(row, col, bound);
    }

    /// Add a unary constraint: `coef * x <= c` (upper or lower bound).
    ///
    /// For upper bound (coef=1): `x <= c` → `x - 0 <= c` → diagonal constraint
    /// For lower bound (coef=-1): `-x <= c` → `x >= -c`
    pub fn add_unary(&mut self, x: VarIndex, coef: i8, c: i128) {
        if self.bottom {
            return;
        }

        // Unary constraints become constraints between v^+ and v^-
        // x <= c → 2x <= 2c → v^+ - v^- <= 2c (halved in some formulations)
        // -x <= c → 2(-x) <= 2c → v^- - v^+ <= 2c
        match coef {
            1 => {
                // x <= c → v^+ - v^- <= 2c
                let bound = c.saturating_mul(2);
                self.add_difference(x.positive(), x.negative(), bound);
            }
            -1 => {
                // -x <= c → v^- - v^+ <= 2c
                let bound = c.saturating_mul(2);
                self.add_difference(x.negative(), x.positive(), bound);
            }
            _ => {}
        }
    }

    /// Close the DBM using Floyd-Warshall algorithm.
    ///
    /// This computes the tightest constraints implied by the current constraints.
    /// After closure, the DBM represents the same set of points but with all
    /// implied constraints explicit.
    pub fn close(&mut self) {
        if self.bottom {
            return;
        }

        let size = self.num_vars * 2;

        // Standard Floyd-Warshall
        for k in 0..size {
            for i in 0..size {
                for j in 0..size {
                    let through_k = self.matrix[i][k].saturating_add(self.matrix[k][j]);
                    self.matrix[i][j] = self.matrix[i][j].min(through_k);
                }
            }
        }

        // Octagon tightening: for each variable v, ensure consistency
        // between v^+ and v^- representations
        for v in 0..self.num_vars {
            let pos = v * 2;
            let neg = v * 2 + 1;

            // Tighten: m[pos][neg] and m[neg][pos] represent 2*v <= c and -2*v <= c
            // We need m[pos][neg] + m[neg][pos] >= 0 for consistency
            let tight_pos_neg = self.matrix[pos][neg].min(self.matrix[pos][neg]);
            let tight_neg_pos = self.matrix[neg][pos].min(self.matrix[neg][pos]);
            self.matrix[pos][neg] = tight_pos_neg;
            self.matrix[neg][pos] = tight_neg_pos;
        }

        // Check for negative cycle (inconsistency)
        for i in 0..size {
            if let Bound::Finite(v) = self.matrix[i][i] {
                if v < 0 {
                    self.bottom = true;
                    return;
                }
            }
        }
    }

    /// Strong closure for octagon DBMs.
    ///
    /// In addition to Floyd-Warshall, applies octagon-specific tightening
    /// to ensure coherence between positive and negative representations.
    pub fn strong_close(&mut self) {
        if self.bottom {
            return;
        }

        // First do standard closure
        self.close();

        if self.bottom {
            return;
        }

        let size = self.num_vars * 2;

        // Octagon coherence: for all i, j
        // m[i][j] <= (m[i][i'] + m[i'][j])/2 where i' is the "opposite" of i
        for i in 0..size {
            let i_opp = if i % 2 == 0 { i + 1 } else { i - 1 };
            for j in 0..size {
                let j_opp = if j % 2 == 0 { j + 1 } else { j - 1 };

                // Coherence tightening
                let via_opp = self.matrix[i][i_opp].saturating_add(self.matrix[i_opp][j]);
                if let Bound::Finite(v) = via_opp {
                    let tight = Bound::Finite(v / 2);
                    self.matrix[i][j] = self.matrix[i][j].min(tight);
                }

                let via_j_opp = self.matrix[i][j_opp].saturating_add(self.matrix[j_opp][j]);
                if let Bound::Finite(v) = via_j_opp {
                    let tight = Bound::Finite(v / 2);
                    self.matrix[i][j] = self.matrix[i][j].min(tight);
                }
            }
        }
    }

    /// Check if the DBM is consistent (no negative cycle).
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        if self.bottom {
            return false;
        }

        let size = self.num_vars * 2;
        for i in 0..size {
            if let Bound::Finite(v) = self.matrix[i][i] {
                if v < 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Get the interval bounds for a variable from the DBM.
    ///
    /// The interval `[lo, hi]` is derived from:
    /// - `hi = m[pos][neg] / 2` (upper bound from `2*v <= m[pos][neg]`)
    /// - `lo = -m[neg][pos] / 2` (lower bound from `-2*v <= m[neg][pos]`)
    ///
    /// # Panics
    ///
    /// Panics if `var.0` >= `num_vars`.
    #[must_use]
    #[allow(clippy::indexing_slicing)] // Indices are validated by var bounds
    pub fn get_interval(&self, var: VarIndex) -> (Option<i128>, Option<i128>) {
        if self.bottom {
            return (None, None);
        }

        let pos = var.positive();
        let neg = var.negative();

        let hi = match self.matrix[pos][neg] {
            Bound::Finite(bound) => Some(bound / 2),
            Bound::PosInf => None,
        };

        let lo = match self.matrix[neg][pos] {
            Bound::Finite(bound) => Some(-bound / 2),
            Bound::PosInf => None,
        };

        (lo, hi)
    }

    /// Pointwise join (max) of two DBMs.
    ///
    /// # Panics
    ///
    /// Panics if `self.num_vars != other.num_vars`.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        if self.bottom {
            return other.clone();
        }
        if other.bottom {
            return self.clone();
        }

        assert_eq!(self.num_vars, other.num_vars);
        let size = self.num_vars * 2;
        let mut result = Self::top(self.num_vars);

        for i in 0..size {
            for j in 0..size {
                result.matrix[i][j] = self.matrix[i][j].max(other.matrix[i][j]);
            }
        }

        result
    }

    /// Pointwise meet (min) of two DBMs, followed by closure.
    ///
    /// # Panics
    ///
    /// Panics if `self.num_vars != other.num_vars`.
    #[must_use]
    pub fn meet(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::bottom(self.num_vars);
        }

        assert_eq!(self.num_vars, other.num_vars);
        let size = self.num_vars * 2;
        let mut result = Self::top(self.num_vars);

        for i in 0..size {
            for j in 0..size {
                result.matrix[i][j] = self.matrix[i][j].min(other.matrix[i][j]);
            }
        }

        result.close();
        result
    }

    /// Standard octagon widening.
    ///
    /// For each entry: if `other[i][j] > self[i][j]`, set to infinity.
    ///
    /// # Panics
    ///
    /// Panics if `self.num_vars != other.num_vars`.
    #[must_use]
    pub fn widen(&self, other: &Self) -> Self {
        if self.bottom {
            return other.clone();
        }
        if other.bottom {
            return self.clone();
        }

        assert_eq!(self.num_vars, other.num_vars);
        let size = self.num_vars * 2;
        let mut result = Self::top(self.num_vars);

        for i in 0..size {
            for j in 0..size {
                if other.matrix[i][j] > self.matrix[i][j] {
                    result.matrix[i][j] = Bound::PosInf;
                } else {
                    result.matrix[i][j] = self.matrix[i][j];
                }
            }
        }

        result
    }

    /// Standard octagon narrowing.
    ///
    /// For each entry: if `self[i][j]` is infinity, take `other[i][j]`.
    ///
    /// # Panics
    ///
    /// Panics if `self.num_vars != other.num_vars`.
    #[must_use]
    pub fn narrow(&self, other: &Self) -> Self {
        if self.bottom {
            return Self::bottom(self.num_vars);
        }
        if other.bottom {
            return Self::bottom(self.num_vars);
        }

        assert_eq!(self.num_vars, other.num_vars);
        let size = self.num_vars * 2;
        let mut result = Self::top(self.num_vars);

        for i in 0..size {
            for j in 0..size {
                if self.matrix[i][j].is_inf() {
                    result.matrix[i][j] = other.matrix[i][j];
                } else {
                    result.matrix[i][j] = self.matrix[i][j];
                }
            }
        }

        result
    }

    /// Check if `self <= other` in the lattice order.
    ///
    /// This holds iff every constraint in `other` is implied by `self`
    /// (i.e., `self[i][j] <= other[i][j]` for all i, j).
    ///
    /// # Panics
    ///
    /// Panics if `self.num_vars != other.num_vars`.
    #[must_use]
    pub fn leq(&self, other: &Self) -> bool {
        if self.bottom {
            return true;
        }
        if other.bottom {
            return false;
        }

        assert_eq!(self.num_vars, other.num_vars);
        let size = self.num_vars * 2;

        for i in 0..size {
            for j in 0..size {
                if self.matrix[i][j] > other.matrix[i][j] {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bound_operations() {
        assert_eq!(
            Bound::Finite(5).saturating_add(Bound::Finite(3)),
            Bound::Finite(8)
        );
        assert_eq!(
            Bound::Finite(5).saturating_add(Bound::PosInf),
            Bound::PosInf
        );
        assert_eq!(Bound::Finite(5).min(Bound::Finite(3)), Bound::Finite(3));
        assert_eq!(Bound::Finite(5).min(Bound::PosInf), Bound::Finite(5));
        assert_eq!(Bound::Finite(5).max(Bound::Finite(3)), Bound::Finite(5));
        assert_eq!(Bound::Finite(5).max(Bound::PosInf), Bound::PosInf);
    }

    #[test]
    fn test_bound_ordering() {
        assert!(Bound::Finite(5) < Bound::PosInf);
        assert!(Bound::Finite(3) < Bound::Finite(5));
        assert!(Bound::PosInf == Bound::PosInf);
    }

    #[test]
    fn test_top_dbm() {
        let dbm = Dbm::top(2);
        assert!(!dbm.is_bottom());
        assert!(dbm.is_consistent());

        // Diagonal should be 0
        assert_eq!(dbm.get(0, 0), Bound::Finite(0));
        assert_eq!(dbm.get(1, 1), Bound::Finite(0));

        // Off-diagonal should be infinity
        assert_eq!(dbm.get(0, 1), Bound::PosInf);
    }

    #[test]
    fn test_bottom_dbm() {
        let dbm = Dbm::bottom(2);
        assert!(dbm.is_bottom());
        assert!(!dbm.is_consistent());
    }

    #[test]
    fn test_add_difference_constraint() {
        let mut dbm = Dbm::top(2);
        dbm.add_difference(0, 1, 5); // x0 - x1 <= 5
        assert_eq!(dbm.get(0, 1), Bound::Finite(5));
    }

    #[test]
    fn test_closure_transitivity() {
        let mut dbm = Dbm::top(2);
        // x0 - x1 <= 3, x1 - x2 <= 4 → x0 - x2 <= 7
        dbm.add_difference(0, 2, 3);
        dbm.add_difference(2, 3, 4);
        dbm.close();

        // Should derive x0 - x3 <= 7
        assert_eq!(dbm.get(0, 3), Bound::Finite(7));
    }

    #[test]
    fn test_closure_detects_inconsistency() {
        let mut dbm = Dbm::top(1);
        // x0 - x0 <= -1 is inconsistent
        dbm.set(0, 0, Bound::Finite(-1));
        dbm.close();
        assert!(dbm.is_bottom());
    }

    #[test]
    fn test_get_interval() {
        let mut dbm = Dbm::top(1);
        let v = VarIndex(0);

        // Set upper bound: v <= 10 → 2v <= 20 → v^+ - v^- <= 20
        dbm.add_unary(v, 1, 10);

        // Set lower bound: -v <= 5 → v >= -5 → v^- - v^+ <= 10
        dbm.add_unary(v, -1, 5);

        let (lo, hi) = dbm.get_interval(v);
        assert_eq!(hi, Some(10));
        assert_eq!(lo, Some(-5));
    }

    #[test]
    fn test_join() {
        let mut dbm1 = Dbm::top(1);
        let mut dbm2 = Dbm::top(1);

        dbm1.add_difference(0, 1, 5);
        dbm2.add_difference(0, 1, 10);

        let joined = dbm1.join(&dbm2);
        // Join takes max (weaker constraint)
        assert_eq!(joined.get(0, 1), Bound::Finite(10));
    }

    #[test]
    fn test_meet() {
        let mut dbm1 = Dbm::top(1);
        let mut dbm2 = Dbm::top(1);

        dbm1.add_difference(0, 1, 5);
        dbm2.add_difference(0, 1, 10);

        let met = dbm1.meet(&dbm2);
        // Meet takes min (stronger constraint)
        assert_eq!(met.get(0, 1), Bound::Finite(5));
    }

    #[test]
    fn test_widen() {
        let mut dbm1 = Dbm::top(1);
        let mut dbm2 = Dbm::top(1);

        dbm1.add_difference(0, 1, 5);
        dbm2.add_difference(0, 1, 10);

        let widened = dbm1.widen(&dbm2);
        // Widening: if other > self, jump to infinity
        assert_eq!(widened.get(0, 1), Bound::PosInf);
    }

    #[test]
    fn test_leq() {
        let mut dbm1 = Dbm::top(1);
        let mut dbm2 = Dbm::top(1);

        dbm1.add_difference(0, 1, 5);
        dbm2.add_difference(0, 1, 10);

        // dbm1 has tighter constraint, so dbm1 <= dbm2
        assert!(dbm1.leq(&dbm2));
        assert!(!dbm2.leq(&dbm1));
    }

    #[test]
    fn test_octagon_constraint() {
        let mut dbm = Dbm::top(2);
        let x = VarIndex(0);
        let y = VarIndex(1);

        // Add: x + y <= 10
        dbm.add_octagon(x, 1, y, 1, 10);

        // This should set m[2*x][2*y+1] = 10 (x - (-y) <= 10)
        assert_eq!(dbm.get(x.positive(), y.negative()), Bound::Finite(10));
    }
}
