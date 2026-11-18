# Inspect image regions in the 2D Viewer

### Related

Requested by Donavan for the CSCI 360 final project.
No upstream issue or maintainer agreement is claimed.
This file is a proposed PR description; no PR has been opened.

### What

The fixed-size pixel hover inspector cannot retain an arbitrary rectangular image region while navigating.
A **Region** toggle in the 2D view title bar now assigns primary drags to an image region inspector.
The selection stores whole-pixel, half-open bounds on an image entity and resolution, clamps reverse/outside drags, and projects its outline through the existing view and image transforms each frame.
A movable crop window reuses the existing colormapped GPU texture at five UI points per texel, with scrolling for large regions, pixel dimensions, coordinate ranges, and a clear button.
Turn Region off to resume navigation while retaining the selected pixels.
Clear the region to restore ordinary entity picking and pixel hover inspection.

The new module keeps geometry, selection lifecycle, and crop presentation together.
Existing spatial-view state stores the per-view selection; existing 2D UI code routes gestures, preserves ordinary navigation by default, and paints the inspector after the scene.
The title-bar hook avoids changing canvas layout.
User documentation, 2D help snapshots, raw/PNG UI snapshots before and after navigation, and a feature changeset accompany the code.
The control appears for views displaying images; it remains available while the mode is enabled even if the image disappears.
The initial architecture design and rejected alternatives were committed before implementation in [DESIGN.md](DESIGN.md).

### Validation

Run from the repository root, using the configured Pixi environment and a graphics-capable session:

```sh
pixi run rs-fmt
pixi run cargo clippy -p re_view_spatial --all-features --all-targets -- -D warnings
pixi run cargo nextest run --all-features --no-fail-fast -p re_view_spatial -j 1 -E 'test(image_region) | test(ui_2d::test_help_view)'
pixi run cargo nextest run --all-features --no-fail-fast -p re_view_spatial
pixi run cargo nextest run --all-targets --all-features --no-fail-fast
pixi run fast-lint
pixi run rerun-build
```

The three new geometry tests, raw RGB and decoded PNG interaction tests, and updated help test are covered by the focused command above.
The final single-worker focused run reported **6 passed, 135 skipped**.
The UI tests log raw RGB and encoded PNG images, enable the real title-bar control, drag through the spatial-view UI, check exact pixel bounds, render reference snapshots before and after pan/zoom input events, check retained bounds, and clear the crop.
Clippy with warnings denied, Rust formatting, and the native Viewer build passed.
The existing Python suite (`pixi run py-test-no-build`, using the existing SDK environment) reported **1,558 passed, 9 skipped, 39 snapshots passed**.

The affected crate run reported **138 passed (1 leaky), 3 failed, 0 skipped**.
All three failing tests were reproduced on unchanged `main` (`151102da35`): `partial_view_visibility::test_partial_view_visibility_2d` (1,024 differing pixels), `voxel_grid_map::test_voxel_grid_map_transparent_opacity_snapshot` (59), and `volume_3d::test_volume_3d_snapshot` (1,994).
Their references were preserved.
Nine existing whole-Viewer references were refreshed only in image-view title bars using actual rendered Region controls, preserving unrelated baseline raster differences.
The C++ task (`pixi run -e cpp cpp-test`) could not configure: `Could NOT find Threads (missing: Threads_FOUND)`; the configured linker rejected the Xcode SDK `.tbd` files with `unknown architecture`, leaving pthread symbols unresolved.
The final full-workspace run reported **3,431 passed, 41 failed, 3 skipped** (3,472 tests run).
Fast lint passed on the staged diff, including codegen verification.
Complete command/output, final lint output, and a representative unchanged-baseline comparison are recorded on the [assignment wiki](https://github.com/donnyboy72/rerun/wiki/Final-Project:-Code-Complete).
The entire existing suite is not claimed to pass.

The real [before recording](https://raw.githubusercontent.com/wiki/donnyboy72/rerun/assets/image-region-before.mp4) was captured before implementation.
The real [after recording](https://raw.githubusercontent.com/wiki/donnyboy72/rerun/assets/image-region-after.mp4) uses the same saved RGB image and comparable Viewer interactions.
The user supplied the native rectangle drag after native drag automation failed; observed crop bounds were 67 × 68 pixels, `x [57, 124), y [53, 121)`.
Native navigation retained the aligned outline and bounds; Clear restored normal hover inspection.
Zoom is additionally exercised by the system tests.
The silent recordings encode timestamped actual Viewer screenshots at low frame rate, with idle waiting trimmed from the finished demonstration.

### Reproduce the demo data

Use the repository's Python SDK environment to save a small deterministic RGB recording:

```python
import numpy as np
import rerun as rr
import rerun.blueprint as rrb

x, y = np.meshgrid(np.arange(320), np.arange(240))
image = np.stack(
    (x * 255 // 319, y * 255 // 239, ((x // 20 + y // 20) % 2) * 180 + 40), axis=-1
).astype(np.uint8)
rr.init("rerun_example_image_region_inspector")
rr.save("image-region-demo.rrd")
rr.log("image", rr.Image(image), static=True)
rr.send_blueprint(rrb.Blueprint(rrb.Spatial2DView(origin="image"), collapse_panels=True))
```

Open the resulting file using `target/debug/rerun --bind 127.0.0.1 --port auto --window-size 1100x760 image-region-demo.rrd`.
`--port auto` ensures the CLI starts this build rather than forwarding the recording to an unrelated running Viewer.
Enable Region, drag over the checkerboard, read its crop dimensions, disable Region, and navigate to verify the outline stays attached.

### Deliberate limits and review focus

No statistics, frame freezing, image editing/export, or persisted blueprint regions.
PNG/JPEG and encoded depth images share the existing decoded-texture path; changing images and video textures simply follow the displayed frame, with no new playback controls.
Only image planes at z = 0 with finite, invertible in-plane transforms are supported.
Selections follow the current image on the same entity and resolution; disappearance or a resolution change clears the region.
Recordings, SDK logging APIs, and blueprint schemas are unchanged.
Adding private state to exported `SpatialViewState` affects external Rust callers constructing it through struct literals.
Review gesture ownership (ordinary picking is suppressed while Region or a retained region is active), that source-compatibility trade-off, and image identity when several samples of one entity are displayed.
The changeset still needs a permanent website-hosted visual asset to meet the upstream release-media convention; the assignment recordings are hosted in the fork wiki.

### Agent

Implemented and documented with OpenAI Codex.
The test evidence is described above; student review and understanding are not asserted on the student's behalf.
No pull request, issue, or maintainer message was sent.
