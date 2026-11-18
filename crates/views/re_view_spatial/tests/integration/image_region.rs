use std::cell::Cell;

use re_log_types::TimePoint;
use re_sdk_types::{RowId, archetypes};
use re_test_context::TestContext;
use re_test_context::external::egui_kittest::kittest::Queryable as _;
use re_test_viewport::TestContextExt as _;
use re_view_spatial::SpatialView2D;
use re_viewer_context::ViewClass as _;
use re_viewport_blueprint::ViewBlueprint;

#[test]
fn image_region_drag_preview_pan_and_clear() {
    let mut test_context = TestContext::new_with_view_class::<SpatialView2D>();
    test_context.log_entity("image", |builder| {
        builder.with_archetype(
            RowId::new(),
            TimePoint::STATIC,
            &archetypes::Image::from_color_model_and_tensor(
                re_sdk_types::encodings::ColorModel::RGB,
                ndarray::Array3::<u8>::from_shape_fn((80, 100, 3), |(y, x, c)| {
                    [x as u8 * 2, y as u8 * 3, 128][c]
                }),
            )
            .unwrap(),
        )
    });
    let view_id = test_context.setup_viewport_blueprint(|_, blueprint| {
        blueprint.add_view_at_root(ViewBlueprint::new_with_root_wildcard(
            SpatialView2D::identifier(),
        ))
    });
    let canvas = Cell::new(egui::Rect::NOTHING);
    let mut harness = test_context
        .setup_kittest_for_rendering_3d(egui::vec2(800.0, 650.0))
        .build_ui(|ui| {
            test_context.run_ui(ui, |ctx, ui| {
                ui.horizontal(|ui| {
                    let mut states = test_context.view_states.lock();
                    let state = states.get_mut_or_create(ctx.store_id(), view_id, &SpatialView2D);
                    SpatialView2D
                        .extra_title_bar_ui(ctx, ui, state, &"/".into(), view_id)
                        .unwrap();
                });
                canvas.set(ui.available_rect_before_wrap());
                test_context.ui_for_single_view(ui, ctx, view_id);
            });
            test_context.handle_system_commands(ui.ctx());
        });
    harness.run();
    harness.get_by_label("Region").click();
    harness.run();

    // Derive pointer positions from the displayed image's letterboxed bounds.
    let rect = canvas.get();
    let scale = (rect.width() / 100.0).min(rect.height() / 80.0);
    let image_min = rect.center() - egui::vec2(100.0, 80.0) * scale * 0.5;
    let start = image_min + egui::vec2(20.25, 20.25) * scale;
    let end = image_min + egui::vec2(59.75, 49.75) * scale;
    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    harness.hover_at(end);
    harness.run();
    harness.drop_at(end);
    harness.run();
    harness.get_by_label("40 × 30 pixels");
    harness.get_by_label("x [20, 60), y [20, 50)");

    harness.snapshot("image_region_inspector");

    // Navigation resumes through the same UI toggle without discarding the region.
    harness.get_by_label("Region").click();
    harness.run();
    let pan_start = image_min + egui::vec2(5.0, 70.0) * scale;
    harness.hover_at(pan_start);
    harness.drag_at(pan_start);
    harness.step();
    harness.hover_at(pan_start + egui::vec2(30.0, 20.0));
    harness.run();
    harness.drop_at(pan_start + egui::vec2(30.0, 20.0));
    harness.run();
    harness.get_by_label("x [20, 60), y [20, 50)");
    harness.hover_at(pan_start);
    harness.event(egui::Event::Zoom(1.5));
    harness.run();
    harness.get_by_label("x [20, 60), y [20, 50)");
    harness.get_by_label("Clear region").click();
    harness.run();
    assert!(
        harness.query_by_label("40 × 30 pixels").is_none(),
        "cleared crop must disappear"
    );
}
