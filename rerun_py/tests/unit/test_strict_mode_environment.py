from __future__ import annotations

import os
import subprocess
import sys

import pytest
from rerun import error_utils


@pytest.mark.parametrize("value", ["0", "false", "OFF", "No", "1", "TRUE", "on", "Yes"])
def test_default_strict_mode_when_recognized_value_returns_corresponding_boolean(
    monkeypatch: pytest.MonkeyPatch, value: str
) -> None:
    monkeypatch.setenv("RERUN_STRICT", value)
    assert error_utils.default_strict_mode() is (value.lower() in {"1", "true", "on", "yes"})


def test_default_strict_mode_when_unset_returns_false_even_if_current_mode_is_true(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("RERUN_STRICT", raising=False)
    monkeypatch.setattr(error_utils, "_strict_mode", True)
    assert error_utils.default_strict_mode() is False


@pytest.mark.parametrize("current", [False, True])
@pytest.mark.parametrize("value", ["", "invalid", " true "])
def test_default_strict_mode_when_invalid_reports_value_and_keeps_current_mode(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], *, current: bool, value: str
) -> None:
    monkeypatch.setenv("RERUN_STRICT", value)
    monkeypatch.setattr(error_utils, "_strict_mode", current)
    assert error_utils.default_strict_mode() is current
    assert capsys.readouterr().out == (
        f"Expected RERUN_STRICT to be one of 0/1 false/true off/on no/yes, found {value.lower()}\n"
    )


@pytest.mark.parametrize("value", ["invalid", "", " true "])
def test_import_when_strict_environment_is_invalid_reports_value_and_defaults_false(value: str) -> None:
    # A fresh process is essential: the current interpreter already initialized _strict_mode.
    result = subprocess.run(
        [sys.executable, "-c", "import rerun; print('strict_mode=' + str(rerun.strict_mode()))"],
        env={**os.environ, "RERUN_STRICT": value},
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert result.stdout.splitlines() == [
        f"Expected RERUN_STRICT to be one of 0/1 false/true off/on no/yes, found {value}",
        "strict_mode=False",
    ]
