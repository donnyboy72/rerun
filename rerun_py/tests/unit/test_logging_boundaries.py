from __future__ import annotations

import logging
from argparse import ArgumentParser, Namespace
from unittest.mock import Mock

import pyarrow as pa
import pytest
import rerun as rr
from rerun import _log
from rerun._logging_handler import LoggingHandler
from rerun._script_helpers import script_add_args, script_setup
from rerun._send_columns import build_column_args


def test_column_args_when_lengths_match_preserves_timeline_and_component_data() -> None:
    time = rr.TimeColumn("frame", sequence=[0, 1])
    column = rr.ComponentColumn("confidence", rr.components.ScalarBatch([0.25, 0.75]))
    timelines, components = build_column_args(iter([time]), iter([column]))
    assert timelines == {"frame": pa.array([0, 1], type=pa.int64())}
    assert components[column.component_descriptor()].to_pylist() == [[0.25], [0.75]]


def test_column_args_when_empty_returns_empty_mappings() -> None:
    assert build_column_args([], []) == ({}, {})
    timelines, _ = build_column_args([rr.TimeColumn("frame", sequence=[])], [])
    assert timelines["frame"].type == pa.int64()
    assert len(timelines["frame"]) == 0


def test_column_args_when_only_components_exist_uses_first_component_length() -> None:
    column = rr.ComponentColumn("value", rr.components.ScalarBatch([1, 2]))
    timelines, components = build_column_args([], [column])
    assert timelines == {}
    assert components[column.component_descriptor()].to_pylist() == [[1.0], [2.0]]


def test_column_args_when_timeline_lengths_differ_reports_offending_timeline() -> None:
    with pytest.raises(ValueError, match="Expected 2 but timeline 'time' has 1"):
        build_column_args([rr.TimeColumn("frame", sequence=[0, 1]), rr.TimeColumn("time", sequence=[0])], [])


def test_column_args_when_component_length_differs_reports_offending_component() -> None:
    column = rr.ComponentColumn("confidence", rr.components.ScalarBatch([1]))
    with pytest.raises(ValueError, match=r"Expected 2 but component .*confidence.* has 1"):
        build_column_args([rr.TimeColumn("frame", sequence=[0, 1])], [column])


def test_handler_when_record_is_formatted_forwards_path_text_and_level(monkeypatch: pytest.MonkeyPatch) -> None:
    log = Mock()
    monkeypatch.setattr(_log, "log", log)
    record = logging.LogRecord("camera", logging.WARNING, "camera.py", 12, "frame %d", (7,), None)
    record.module = "sensors.camera"
    LoggingHandler("logs").emit(record)
    log.assert_called_once()
    path, text_log = log.call_args.args
    assert path == "logs/sensors/camera"
    assert text_log.text.as_arrow_array().to_pylist() == ["frame 7"]
    assert text_log.level.as_arrow_array().to_pylist() == ["WARN"]


def test_handler_when_custom_level_and_no_prefix_preserves_custom_level(monkeypatch: pytest.MonkeyPatch) -> None:
    log = Mock()
    monkeypatch.setattr(_log, "log", log)
    record = logging.LogRecord("camera", 25, "camera.py", 1, "", (), None)
    record.levelname = "NOTICE"
    LoggingHandler().emit(record)
    path, text_log = log.call_args.args
    assert path == "camera"
    assert text_log.text.as_arrow_array().to_pylist() == [""]
    assert text_log.level.as_arrow_array().to_pylist() == ["NOTICE"]


def test_handler_when_formatting_fails_does_not_forward_record(monkeypatch: pytest.MonkeyPatch) -> None:
    log = Mock()
    monkeypatch.setattr(_log, "log", log)
    record = logging.LogRecord("camera", logging.INFO, "camera.py", 1, "%d", ("bad",), None)
    with pytest.raises(TypeError):
        LoggingHandler().emit(record)
    log.assert_not_called()


@pytest.fixture
def script_recording(monkeypatch: pytest.MonkeyPatch) -> Mock:
    recording = Mock()
    monkeypatch.setattr(rr, "init", Mock())
    monkeypatch.setattr(rr, "get_global_data_recording", Mock(return_value=recording))
    monkeypatch.setattr(rr, "serve_web_viewer", Mock())
    return recording


def script_args(*flags: str) -> Namespace:
    parser = ArgumentParser()
    script_add_args(parser)
    return parser.parse_args(flags)


@pytest.mark.parametrize("flag,method", [("--stdout", "stdout"), ("--connect", "connect_grpc"), ("--save", "save")])
def test_script_setup_when_sink_selected_routes_recording(script_recording: Mock, flag: str, method: str) -> None:
    flags = [flag, "capture.rrd"] if flag == "--save" else [flag]
    blueprint = rr.blueprint.Blueprint()
    assert (
        script_setup(script_args(*flags), "test_app", recording_id="test_id", default_blueprint=blueprint)
        is script_recording
    )
    rr.init.assert_called_once_with(
        application_id="test_app", recording_id="test_id", default_enabled=True, strict=True
    )
    positional = ("capture.rrd",) if flag == "--save" else (None,) if flag == "--connect" else ()
    getattr(script_recording, method).assert_called_once_with(*positional, default_blueprint=blueprint)
    script_recording.spawn.assert_not_called()


def test_script_setup_when_headless_does_not_create_sink(script_recording: Mock) -> None:
    assert script_setup(script_args("--headless"), "test_app") is script_recording
    assert script_recording.mock_calls == []


def test_script_setup_when_default_spawns_viewer(script_recording: Mock) -> None:
    script_setup(script_args(), "test_app")
    script_recording.spawn.assert_called_once_with(default_blueprint=None)


def test_script_setup_when_serving_connects_web_viewer_to_grpc(script_recording: Mock) -> None:
    script_recording.serve_grpc.return_value = "rerun+http://localhost:9876/proxy"
    script_setup(script_args("--serve"), "test_app")
    script_recording.serve_grpc.assert_called_once_with(default_blueprint=None)
    rr.serve_web_viewer.assert_called_once_with(open_browser=True, connect_to=script_recording.serve_grpc.return_value)


def test_script_setup_when_flags_conflict_stdout_takes_priority(script_recording: Mock) -> None:
    script_setup(script_args("--stdout", "--serve", "--connect", "--save", "capture.rrd"), "test_app")
    script_recording.stdout.assert_called_once_with(default_blueprint=None)
    assert len(script_recording.mock_calls) == 1
    rr.serve_web_viewer.assert_not_called()


def test_script_setup_when_connection_fails_propagates_error_without_spawning(script_recording: Mock) -> None:
    script_recording.connect_grpc.side_effect = RuntimeError("connection unavailable")
    with pytest.raises(RuntimeError, match="connection unavailable"):
        script_setup(script_args("--connect", "--url", "rerun+http://localhost:1/proxy"), "test_app")
    script_recording.connect_grpc.assert_called_once_with("rerun+http://localhost:1/proxy", default_blueprint=None)
    script_recording.spawn.assert_not_called()
