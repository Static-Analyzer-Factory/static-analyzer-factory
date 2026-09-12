#!/usr/bin/env python3
"""Tests for the SV-COMP 2027 witness-requirement rule (`svcomp_witness_rules`).

These pin the rows of https://sv-comp.sosy-lab.org/2027/rules.php that SAF's score
depends on. When the next edition lands, these are the tests that must go red.
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from svcomp_witness_rules import (  # noqa: E402
    BASE_CATEGORY_SETS, DEMO_CATEGORIES, base_categories, false_witness_requirement,
    sniff_witness_format, true_witness_requirement, version_satisfies,
)


class TestBaseCategoryMapping(unittest.TestCase):
    """§1 — the suffix is the 2027 base category, NOT the `.set` basename."""

    def test_linkedlists_unreach_call_is_the_heap_category(self):
        # There is no `C.unreach-call.LinkedLists`. Getting this wrong makes a
        # witness-FREE category look witness-REQUIRED.
        self.assertEqual(base_categories("unreach-call", {"LinkedLists"}), {"Heap"})

    def test_linkedlists_memsafety_is_its_own_category(self):
        # ...but for valid-memsafety, LinkedLists IS a suffix. Hence (property, set).
        self.assertEqual(base_categories("valid-memsafety", {"LinkedLists"}), {"LinkedLists"})

    def test_one_set_maps_to_different_suffixes_per_property(self):
        self.assertEqual(base_categories("unreach-call", {"Heap"}), {"Heap"})
        self.assertEqual(base_categories("no-overflow", {"Heap"}), {"Main"})
        self.assertEqual(base_categories("termination", {"Heap"}), {"Other"})
        self.assertEqual(base_categories("valid-memsafety", {"Heap"}), {"Heap"})
        self.assertEqual(base_categories("valid-memcleanup", {"Heap"}), {"Main"})

    def test_termination_has_no_literal_all_suffix(self):
        # The rules table's "all" row is a wildcard over every real suffix.
        self.assertNotIn("all", BASE_CATEGORY_SETS["termination"])
        self.assertEqual(
            set(BASE_CATEGORY_SETS["termination"]),
            {"BitVectors", "MainControlFlow", "MainHeap", "Other",
             "SoftwareSystems-DeviceDriversLinux64", "SoftwareSystems-uthash"})

    def test_task_can_sit_in_two_base_categories(self):
        self.assertEqual(base_categories("unreach-call", {"Concurrency", "Huawei-Concurrency-Challenges"}),
                         {"Concurrency", "Huawei-Concurrency-Challenges"})

    def test_out_of_competition_sets_have_no_base_category(self):
        for s in ("Unused_Juliet", "Unused_DeviceDriversLinux64Regression"):
            for prop in BASE_CATEGORY_SETS:
                self.assertEqual(base_categories(prop, {s}), set(), f"{prop}/{s}")

    def test_a_task_in_no_set_at_all_has_no_base_category(self):
        self.assertEqual(base_categories("unreach-call", set()), set())


class TestTrueWitnessRule(unittest.TestCase):
    """§2, correctness column."""

    def test_termination_requires_21_this_is_the_2027_change(self):
        # THE regression this module exists to prevent. 2026: "2.1 (demo mode)"
        # => free. 2027: "2.1 or higher" => required. Worth 36 weighted points.
        for suffix in BASE_CATEGORY_SETS["termination"]:
            self.assertEqual(true_witness_requirement("termination", {suffix}), (True, "2.1"),
                             f"termination.{suffix}")

    def test_unreach_call_arrays_and_heap_are_not_supported(self):
        self.assertEqual(true_witness_requirement("unreach-call", {"Arrays"}), (False, None))
        self.assertEqual(true_witness_requirement("unreach-call", {"Heap"}), (False, None))

    def test_unreach_call_floats_is_demo_mode_hence_free(self):
        self.assertEqual(true_witness_requirement("unreach-call", {"Floats"}), (False, None))

    def test_unreach_call_concurrency_requires_21(self):
        self.assertEqual(true_witness_requirement("unreach-call", {"Concurrency"}), (True, "2.1"))

    def test_huawei_is_demo_mode_hence_free(self):
        h = {"Huawei-Concurrency-Challenges"}
        self.assertEqual(true_witness_requirement("unreach-call", h), (False, None))
        self.assertEqual(true_witness_requirement("no-overflow", h), (False, None))

    def test_unreach_call_default_requires_20(self):
        self.assertEqual(true_witness_requirement("unreach-call", {"Loops"}), (True, "2.0"))
        self.assertEqual(true_witness_requirement("unreach-call", {"ECA"}), (True, "2.0"))

    def test_verdict_only_properties_on_the_true_side(self):
        self.assertEqual(true_witness_requirement("valid-memsafety", {"Heap"}), (False, None))
        self.assertEqual(true_witness_requirement("valid-memcleanup", {"Main"}), (False, None))
        self.assertEqual(true_witness_requirement("no-data-race", {"Concurrency"}), (False, None))

    def test_multi_category_rounds_the_requirement_up(self):
        # Concurrency requires 2.1; Huawei is demo/free. Conservative => required.
        self.assertEqual(
            true_witness_requirement("unreach-call", {"Concurrency", "Huawei-Concurrency-Challenges"}),
            (True, "2.1"))

    def test_out_of_competition_falls_back_to_the_generic_cell(self):
        # A task in no 2027 base category is not run by SV-COMP. Scoring it as
        # "no witness required" would hand ~19.9k out-of-competition tasks free
        # points; fall back to the property's generic cell instead.
        self.assertEqual(true_witness_requirement("termination", set()), (True, "2.1"))
        self.assertEqual(true_witness_requirement("unreach-call", set()), (True, "2.0"))
        self.assertEqual(true_witness_requirement("valid-memsafety", set()), (False, None))
        self.assertEqual(false_witness_requirement("unreach-call", set()), (True, "2.0"))


class TestFalseWitnessRule(unittest.TestCase):
    """§2, violation column."""

    def test_no_data_race_violation_is_required_at_22(self):
        # REVERSES the pre-2027 harness, which exempted no-data-race FALSE.
        # Worth -5 weighted. The 2026 page already required format 1.0 here.
        self.assertEqual(false_witness_requirement("no-data-race", {"Concurrency"}), (True, "2.2"))

    def test_memsafety_concurrency_and_huawei_violations_are_demo_hence_free(self):
        self.assertEqual(false_witness_requirement("valid-memsafety", {"Concurrency"}), (False, None))
        self.assertEqual(
            false_witness_requirement("valid-memsafety", {"Huawei-Concurrency-Challenges"}), (False, None))

    def test_memsafety_default_violation_requires_20(self):
        self.assertEqual(false_witness_requirement("valid-memsafety", {"Heap"}), (True, "2.0"))

    def test_memtrack_violations_have_no_supported_format(self):
        # Footnote #: no format covers valid-memtrack, so it scores on the verdict.
        self.assertEqual(
            false_witness_requirement("valid-memsafety", {"Heap"}, "valid-memtrack"), (False, None))
        # ...but valid-deref / valid-free still need one.
        self.assertEqual(
            false_witness_requirement("valid-memsafety", {"Heap"}, "valid-deref"), (True, "2.0"))
        self.assertEqual(
            false_witness_requirement("valid-memsafety", {"Heap"}, "valid-free"), (True, "2.0"))

    def test_concurrency_violations_require_22(self):
        self.assertEqual(false_witness_requirement("unreach-call", {"Concurrency"}), (True, "2.2"))
        self.assertEqual(false_witness_requirement("no-overflow", {"Concurrency"}), (True, "2.2"))

    def test_memcleanup_is_free_on_both_sides(self):
        self.assertEqual(false_witness_requirement("valid-memcleanup", {"Main"}), (False, None))
        self.assertEqual(true_witness_requirement("valid-memcleanup", {"Main"}), (False, None))


class TestVersionFloor(unittest.TestCase):
    def test_meets_and_misses(self):
        self.assertTrue(version_satisfies("2.1", "2.0"))
        self.assertTrue(version_satisfies("2.0", "2.0"))
        self.assertFalse(version_satisfies("2.0", "2.1"))
        self.assertFalse(version_satisfies("2.1", "2.2"))

    def test_no_floor_always_satisfied(self):
        self.assertTrue(version_satisfies(None, None))
        self.assertTrue(version_satisfies("2.0", None))

    def test_unrecorded_format_is_not_penalised(self):
        # Dumps predating the `witness_format` field must re-score identically.
        self.assertTrue(version_satisfies(None, "2.2"))

    def test_graphml_never_satisfies_a_yaml_floor(self):
        self.assertFalse(version_satisfies("graphml-1.0", "2.0"))


class TestSniffWitnessFormat(unittest.TestCase):
    def _write(self, text):
        import tempfile
        fd, path = tempfile.mkstemp(suffix=".yml")
        with os.fdopen(fd, "w") as f:
            f.write(text)
        self.addCleanup(os.unlink, path)
        return path

    def test_yaml_20(self):
        p = self._write("- entry_type: violation_sequence\n  metadata:\n    format_version: '2.0'\n")
        self.assertEqual(sniff_witness_format(p), "2.0")

    def test_yaml_21_unquoted(self):
        p = self._write("- entry_type: invariant_set\n  metadata:\n    format_version: 2.1\n")
        self.assertEqual(sniff_witness_format(p), "2.1")

    def test_graphml(self):
        p = self._write('<?xml version="1.0"?>\n<graphml xmlns="http://graphml.graphdrawing.org/xmlns">\n')
        self.assertEqual(sniff_witness_format(p), "graphml-1.0")

    def test_missing_file(self):
        self.assertIsNone(sniff_witness_format("/nonexistent/w.yml"))
        self.assertIsNone(sniff_witness_format(None))


class TestDemoCategories(unittest.TestCase):
    def test_huawei_scores_do_not_count_toward_overall(self):
        # Distinct from the "(demo mode)" WITNESS column — this is about the SCORE.
        self.assertIn("C.no-data-race.Huawei-Concurrency-Challenges", DEMO_CATEGORIES)
        self.assertIn("C.unreach-call.Huawei-Concurrency-Challenges", DEMO_CATEGORIES)


if __name__ == "__main__":
    unittest.main(verbosity=2)
