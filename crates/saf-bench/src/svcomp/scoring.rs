//! SV-COMP scoring system.
//!
//! Implements the official SV-COMP 2026 scoring rules:
//! - TRUE correct: +2 points (proved property holds when it should)
//! - FALSE correct: +1 point (found real violation)
//! - TRUE incorrect: -32 points (missed bug - unsound)
//! - FALSE incorrect: -16 points (false alarm)
//! - UNKNOWN: 0 points (timeout, crash, limitation)

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::property::PropertyResult;
use super::task::{Property, SvCompTask};

/// Verdict produced by SAF for a property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SvCompVerdict {
    /// Property holds (no violations found).
    True,
    /// Property violated (bug found).
    False {
        /// Optional witness trace.
        witness: Option<Vec<String>>,
    },
    /// Cannot determine (timeout, limitation, etc.).
    Unknown {
        /// Reason for unknown result.
        reason: String,
    },
}

impl From<PropertyResult> for SvCompVerdict {
    fn from(result: PropertyResult) -> Self {
        match result {
            PropertyResult::True => Self::True,
            PropertyResult::False { witness } => Self::False { witness },
            PropertyResult::Unknown { reason } => Self::Unknown { reason },
        }
    }
}

/// Outcome of comparing SAF's verdict against expected verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SvCompOutcome {
    /// TRUE correct: SAF said TRUE and expected was TRUE (+2 points).
    TrueCorrect,
    /// FALSE correct: SAF said FALSE and expected was FALSE (+1 point).
    FalseCorrect,
    /// TRUE incorrect: SAF said TRUE but expected was FALSE (-32 points, unsound).
    TrueIncorrect,
    /// FALSE incorrect: SAF said FALSE but expected was TRUE (-16 points, false alarm).
    FalseIncorrect,
    /// UNKNOWN: SAF couldn't determine (0 points).
    Unknown,
}

impl SvCompOutcome {
    /// Get the SV-COMP score for this outcome.
    pub fn score(&self) -> i32 {
        match self {
            Self::TrueCorrect => 2,
            Self::FalseCorrect => 1,
            Self::TrueIncorrect => -32,
            Self::FalseIncorrect => -16,
            Self::Unknown => 0,
        }
    }

    /// Returns true if this is a correct outcome (positive or zero score).
    pub fn is_correct(&self) -> bool {
        matches!(self, Self::TrueCorrect | Self::FalseCorrect | Self::Unknown)
    }

    /// Returns true if this is an incorrect outcome (negative score).
    pub fn is_incorrect(&self) -> bool {
        matches!(self, Self::TrueIncorrect | Self::FalseIncorrect)
    }

    /// Human-readable label for the outcome.
    pub fn label(&self) -> &'static str {
        match self {
            Self::TrueCorrect => "TRUE (correct)",
            Self::FalseCorrect => "FALSE (correct)",
            Self::TrueIncorrect => "TRUE (INCORRECT)",
            Self::FalseIncorrect => "FALSE (incorrect)",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Compute the outcome by comparing verdict against expected.
pub fn compute_outcome(verdict: &SvCompVerdict, expected: bool) -> SvCompOutcome {
    match (verdict, expected) {
        (SvCompVerdict::True, true) => SvCompOutcome::TrueCorrect,
        (SvCompVerdict::True, false) => SvCompOutcome::TrueIncorrect,
        (SvCompVerdict::False { .. }, false) => SvCompOutcome::FalseCorrect,
        (SvCompVerdict::False { .. }, true) => SvCompOutcome::FalseIncorrect,
        (SvCompVerdict::Unknown { .. }, _) => SvCompOutcome::Unknown,
    }
}

/// Result of analyzing a single task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// The task that was analyzed.
    pub task: SvCompTask,
    /// Property that was checked.
    pub property: Property,
    /// SAF's verdict.
    pub verdict: SvCompVerdict,
    /// Expected verdict from task definition.
    pub expected: Option<bool>,
    /// Computed outcome (if expected is known).
    pub outcome: Option<SvCompOutcome>,
    /// Time taken for analysis.
    pub duration: Duration,
}

impl TaskResult {
    /// Create a new task result.
    pub fn new(
        task: SvCompTask,
        property: Property,
        verdict: SvCompVerdict,
        expected: Option<bool>,
        duration: Duration,
    ) -> Self {
        let outcome = expected.map(|e| compute_outcome(&verdict, e));
        Self {
            task,
            property,
            verdict,
            expected,
            outcome,
            duration,
        }
    }

    /// Get the score for this result.
    pub fn score(&self) -> i32 {
        self.outcome.map_or(0, |o| o.score())
    }

    /// Get maximum possible score for this result.
    pub fn max_score(&self) -> i32 {
        match self.expected {
            Some(true) => 2,  // TRUE correct would give +2
            Some(false) => 1, // FALSE correct would give +1
            None => 0,        // No expected verdict
        }
    }
}

/// Summary of results for a single category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Category name (e.g., "ReachSafety-Arrays").
    pub category: String,
    /// Subcategory name (if applicable).
    pub subcategory: Option<String>,
    /// Property type.
    pub property: String,
    /// Total number of tasks.
    pub total_tasks: usize,
    /// TRUE correct count.
    pub true_correct: usize,
    /// FALSE correct count.
    pub false_correct: usize,
    /// TRUE incorrect count (unsound).
    pub true_incorrect: usize,
    /// FALSE incorrect count (false alarm).
    pub false_incorrect: usize,
    /// UNKNOWN count.
    pub unknown: usize,
    /// Total score.
    pub score: i32,
    /// Maximum possible score.
    pub max_possible_score: i32,
    /// Individual task results.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<TaskResult>,
}

impl CategorySummary {
    /// Create a new category summary.
    pub fn new(category: String, property: String) -> Self {
        Self {
            category,
            property,
            ..Default::default()
        }
    }

    /// Add a task result to this category.
    pub fn add_result(&mut self, result: TaskResult) {
        self.total_tasks += 1;
        self.score += result.score();
        self.max_possible_score += result.max_score();

        if let Some(outcome) = result.outcome {
            match outcome {
                SvCompOutcome::TrueCorrect => self.true_correct += 1,
                SvCompOutcome::FalseCorrect => self.false_correct += 1,
                SvCompOutcome::TrueIncorrect => self.true_incorrect += 1,
                SvCompOutcome::FalseIncorrect => self.false_incorrect += 1,
                SvCompOutcome::Unknown => self.unknown += 1,
            }
        } else {
            self.unknown += 1;
        }

        self.results.push(result);
    }

    /// Calculate the score percentage.
    pub fn score_percentage(&self) -> f64 {
        if self.max_possible_score <= 0 {
            0.0
        } else {
            (f64::from(self.score) / f64::from(self.max_possible_score)) * 100.0
        }
    }

    /// Count of correct results (TRUE correct + FALSE correct).
    pub fn correct_count(&self) -> usize {
        self.true_correct + self.false_correct
    }

    /// Count of incorrect results (TRUE incorrect + FALSE incorrect).
    pub fn incorrect_count(&self) -> usize {
        self.true_incorrect + self.false_incorrect
    }
}

/// Overall summary of SV-COMP benchmark run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SvCompSummary {
    /// Results by category.
    pub categories: Vec<CategorySummary>,
    /// Total score across all categories.
    pub total_score: i32,
    /// Maximum possible score.
    pub max_possible_score: i32,
    /// Total tasks analyzed.
    pub total_tasks: usize,
    /// TRUE correct count.
    pub true_correct: usize,
    /// FALSE correct count.
    pub false_correct: usize,
    /// TRUE incorrect count.
    pub true_incorrect: usize,
    /// FALSE incorrect count.
    pub false_incorrect: usize,
    /// UNKNOWN count.
    pub unknown: usize,
    /// Total time taken.
    pub timing_secs: f64,
}

impl SvCompSummary {
    /// Create a summary from category summaries.
    pub fn from_categories(categories: Vec<CategorySummary>, total_time: Duration) -> Self {
        let mut summary = Self {
            timing_secs: total_time.as_secs_f64(),
            ..Default::default()
        };

        for cat in &categories {
            summary.total_tasks += cat.total_tasks;
            summary.total_score += cat.score;
            summary.max_possible_score += cat.max_possible_score;
            summary.true_correct += cat.true_correct;
            summary.false_correct += cat.false_correct;
            summary.true_incorrect += cat.true_incorrect;
            summary.false_incorrect += cat.false_incorrect;
            summary.unknown += cat.unknown;
        }

        summary.categories = categories;
        summary
    }

    /// Calculate the overall score percentage.
    pub fn score_percentage(&self) -> f64 {
        if self.max_possible_score <= 0 {
            0.0
        } else {
            (f64::from(self.total_score) / f64::from(self.max_possible_score)) * 100.0
        }
    }

    /// Get incorrect task details for reporting.
    pub fn incorrect_details(&self) -> Vec<IncorrectDetail> {
        let mut details = Vec::new();

        for cat in &self.categories {
            for result in &cat.results {
                if let Some(outcome) = result.outcome {
                    if outcome.is_incorrect() {
                        details.push(IncorrectDetail {
                            file: result.task.path.display().to_string(),
                            category: cat.category.clone(),
                            property: result.property.name().to_string(),
                            outcome,
                            expected: result.expected,
                        });
                    }
                }
            }
        }

        details
    }
}

/// Details about an incorrect result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncorrectDetail {
    /// Path to the task file.
    pub file: String,
    /// Category name.
    pub category: String,
    /// Property name.
    pub property: String,
    /// The incorrect outcome.
    pub outcome: SvCompOutcome,
    /// Expected verdict.
    pub expected: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_scores() {
        assert_eq!(SvCompOutcome::TrueCorrect.score(), 2);
        assert_eq!(SvCompOutcome::FalseCorrect.score(), 1);
        assert_eq!(SvCompOutcome::TrueIncorrect.score(), -32);
        assert_eq!(SvCompOutcome::FalseIncorrect.score(), -16);
        assert_eq!(SvCompOutcome::Unknown.score(), 0);
    }

    #[test]
    fn test_compute_outcome() {
        // TRUE verdict, expected TRUE -> correct
        assert_eq!(
            compute_outcome(&SvCompVerdict::True, true),
            SvCompOutcome::TrueCorrect
        );

        // TRUE verdict, expected FALSE -> unsound!
        assert_eq!(
            compute_outcome(&SvCompVerdict::True, false),
            SvCompOutcome::TrueIncorrect
        );

        // FALSE verdict, expected FALSE -> correct (found bug)
        assert_eq!(
            compute_outcome(&SvCompVerdict::False { witness: None }, false),
            SvCompOutcome::FalseCorrect
        );

        // FALSE verdict, expected TRUE -> false alarm
        assert_eq!(
            compute_outcome(&SvCompVerdict::False { witness: None }, true),
            SvCompOutcome::FalseIncorrect
        );

        // UNKNOWN -> always unknown
        assert_eq!(
            compute_outcome(
                &SvCompVerdict::Unknown {
                    reason: "test".into()
                },
                true
            ),
            SvCompOutcome::Unknown
        );
    }

    #[test]
    fn test_category_summary() {
        use std::path::PathBuf;

        let mut summary = CategorySummary::new("test".into(), "unreach-call".into());

        let task = SvCompTask {
            path: PathBuf::from("test.yml"),
            bitcode_path: PathBuf::from("test.bc"),
            input_files: vec![],
            properties: vec![],
            language: super::super::task::Language::C,
            data_model: super::super::task::DataModel::LP64,
            category: "test".into(),
        };

        // Add a TRUE correct result
        summary.add_result(TaskResult::new(
            task.clone(),
            Property::UnreachCall,
            SvCompVerdict::True,
            Some(true),
            Duration::from_secs(1),
        ));

        assert_eq!(summary.total_tasks, 1);
        assert_eq!(summary.true_correct, 1);
        assert_eq!(summary.score, 2);
        assert_eq!(summary.max_possible_score, 2);
        assert_eq!(summary.score_percentage(), 100.0);
    }

    #[test]
    fn test_outcome_is_correct() {
        assert!(SvCompOutcome::TrueCorrect.is_correct());
        assert!(SvCompOutcome::FalseCorrect.is_correct());
        assert!(SvCompOutcome::Unknown.is_correct());
        assert!(!SvCompOutcome::TrueIncorrect.is_correct());
        assert!(!SvCompOutcome::FalseIncorrect.is_correct());
    }
}
