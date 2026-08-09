# This file is part of the SAF (Static Analyzer Factory) SV-COMP integration.
#
# BenchExec tool-info module for SAF, per plan 192 (P0.2).
#
# SAF's `saf verify` subcommand is the blind competition entry point: it prints
# exactly one verdict line on stdout — one of the BenchExec `RESULT_*` strings
# (`true` / `false(<subproperty>)` / `unknown`) — and nothing else (diagnostics
# go to stderr). This module assembles the command line and maps that stdout back
# to a BenchExec result.
#
# SPDX-License-Identifier: Apache-2.0

import benchexec.result as result
import benchexec.tools.template


class Tool(benchexec.tools.template.BaseTool2):
    """Tool-info module for SAF (Static Analyzer Factory)."""

    def executable(self, tool_locator):
        return tool_locator.find_executable("saf")

    def name(self):
        return "SAF"

    def project_url(self):
        return "https://github.com/Static-Analyzer-Factory/static-analyzer-factory"

    def version(self, executable):
        return self._version_from_tool(executable, arg="--version").strip()

    def cmdline(self, executable, options, task, rlimits):
        cmd = [executable, "verify"]

        if task.property_file:
            cmd += ["--property", task.property_file]

        # Data model (ILP32/LP64) comes from the task-definition `options` block.
        data_model = None
        if task.options:
            data_model = task.options.get("data_model")
        if data_model:
            cmd += ["--data-model", data_model]

        # `options` carries the tool's configured benchexec options, including the
        # witness path (`--witness ${witness}`); append them verbatim.
        cmd += list(options)

        # SAF verifies exactly one program file (a `.c` or preprocessed `.i`).
        cmd.append(task.single_input_file)
        return cmd

    def determine_result(self, run):
        # SAF prints exactly one verdict line on stdout; stderr diagnostics never
        # match a RESULT_* string. Scan for the verdict; anything else (crash,
        # timeout, garbage) is UNKNOWN — never a scoring-negative guess.
        for line in run.output:
            verdict = line.strip()
            if verdict == result.RESULT_TRUE_PROP:
                return result.RESULT_TRUE_PROP
            if verdict == result.RESULT_UNKNOWN:
                return result.RESULT_UNKNOWN
            if verdict == result.RESULT_FALSE_PROP or (
                verdict.startswith(result.RESULT_FALSE_PROP + "(") and verdict.endswith(")")
            ):
                # e.g. "false(unreach-call)" == result.RESULT_FALSE_REACH
                return verdict
        return result.RESULT_UNKNOWN
