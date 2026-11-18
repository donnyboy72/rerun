//! Transient image-coordinate selection and presentation for 2D spatial views.

use egui::{Pos2, Rect, Vec2, emath::RectTransform};
use re_log_types::EntityPath;
use re_ui::UiExt as _;
use re_viewer_context::{SystemExecutionOutput, ViewerContext, gpu_bridge};

use crate::{PickableRectSourceData, PickableTexturedRect, visualizers::iter_spatial_data};

/// Whole-pixel bounds, inclusive at `min` and exclusive at `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, re_byte_size::SizeBytes)]
struct PixelRegion {
    min: [u32; 2],
    max: [u32; 2],
}

impl PixelRegion {
    fn from_drag(start: Pos2, end: Pos2, resolution: [u32; 2]) -> Option<Self> {
        if !start.is_finite() || !end.is_finite() || start.x == end.x || start.y == end.y {
            return None;
        }
        let bounds = Rect::from_two_pos(start, end);
        let min = [bounds.min.x.floor(), bounds.min.y.floor()];
        let max = [bounds.max.x.ceil(), bounds.max.y.ceil()];
        let min = std::array::from_fn(|i| min[i].clamp(0.0, resolution[i] as f32) as u32);
        let max = std::array::from_fn(|i| max[i].clamp(0.0, resolution[i] as f32) as u32);
        (min[0] < max[0] && min[1] < max[1]).then_some(Self { min, max })
    }

    fn rect(self) -> Rect {
        Rect::from_min_max(
            egui::pos2(self.min[0] as f32, self.min[1] as f32),
            egui::pos2(self.max[0] as f32, self.max[1] as f32),
        )
    }
}

/// An affine image plane expressed in scene coordinates.
struct ImagePlane {
    origin: glam::Vec2,
    scene_from_image: glam::Mat2,
    image_from_scene: glam::Mat2,
}

impl ImagePlane {
    fn new(origin: glam::Vec2, u: glam::Vec2, v: glam::Vec2) -> Option<Self> {
        let scene_from_image = glam::Mat2::from_cols(u, v);
        let determinant = scene_from_image.determinant();
        if !origin.is_finite()
            || !scene_from_image.is_finite()
            || !determinant.is_finite()
            || determinant == 0.0
        {
            return None;
        }
        let image_from_scene = scene_from_image.inverse();
        image_from_scene.is_finite().then_some(Self {
            origin,
            scene_from_image,
            image_from_scene,
        })
    }

    fn from_image(image: &PickableTexturedRect) -> Option<Self> {
        if !matches!(image.source_data, PickableRectSourceData::Image { .. }) {
            return None;
        }
        let rect = &image.textured_rect;
        let [width, height] = image.resolution();
        if width == 0
            || height == 0
            || rect.top_left_corner_position.z != 0.0
            || rect.extent_u.z != 0.0
            || rect.extent_v.z != 0.0
        {
            return None;
        }
        Self::new(
            rect.top_left_corner_position.truncate(),
            rect.extent_u.truncate() / width as f32,
            rect.extent_v.truncate() / height as f32,
        )
    }

    fn image_from_ui(&self, pos: Pos2, ui_from_scene: RectTransform) -> Pos2 {
        let scene = ui_from_scene.inverse().transform_pos(pos);
        let image = self.image_from_scene * (glam::vec2(scene.x, scene.y) - self.origin);
        egui::pos2(image.x, image.y)
    }

    fn ui_from_image(&self, pos: Pos2, ui_from_scene: RectTransform) -> Pos2 {
        let scene = self.origin + self.scene_from_image * glam::vec2(pos.x, pos.y);
        ui_from_scene.transform_pos(egui::pos2(scene.x, scene.y))
    }
}

#[derive(Clone, re_byte_size::SizeBytes)]
struct Selection {
    entity: EntityPath,
    resolution: [u32; 2],
    start: Pos2,
    region: Option<PixelRegion>,
}

impl Selection {
    fn matches(&self, image: &PickableTexturedRect) -> bool {
        self.entity == image.ent_path && self.resolution == image.resolution()
    }
}

#[derive(Clone, Default, re_byte_size::SizeBytes)]
pub(crate) struct ImageRegionInspector {
    pub enabled: bool,
    selection: Option<Selection>,
    dragging: bool,
}

impl ImageRegionInspector {
    pub fn title_bar_ui(&mut self, ui: &mut egui::Ui) {
        if ui.selectable_label(self.enabled, "Region").on_hover_text(
            "Inspect image region: enable, then drag a rectangle on an image. Disable to pan; the selection stays attached to the image.",
        ).clicked() {
            self.enabled = !self.enabled;
            self.dragging = false;
        }
    }

    pub fn interact(
        &mut self,
        ctx: &ViewerContext<'_>,
        response: &egui::Response,
        ui_from_scene: RectTransform,
        output: &SystemExecutionOutput,
        query: &re_viewer_context::ViewQuery<'_>,
    ) {
        let images = || iter_spatial_data(output).flat_map(|data| &data.pickable_rects);
        if self.selection.as_ref().is_some_and(|selection| {
            !images()
                .any(|image| selection.matches(image) && ImagePlane::from_image(image).is_some())
        }) {
            self.selection = None;
            self.dragging = false;
        }
        if !self.enabled {
            return;
        }
        if response.hovered() {
            response.ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
            if response.ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.selection = None;
                self.dragging = false;
                return;
            }
        }
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(start_ui) = response.ctx.input(|i| i.pointer.press_origin())
        {
            let result = ctx.lookup_query_result(query.view_id);
            let picked = images()
                .filter_map(|image| {
                    if !result
                        .tree
                        .lookup_result_by_path(image.ent_path.hash())?
                        .is_interactive()
                    {
                        return None;
                    }
                    let plane = ImagePlane::from_image(image)?;
                    let start = plane.image_from_ui(start_ui, ui_from_scene);
                    let [width, height] = image.resolution();
                    (start.x >= 0.0
                        && start.y >= 0.0
                        && start.x < width as f32
                        && start.y < height as f32)
                        .then_some((image, start))
                })
                .max_by_key(|(image, _)| image.textured_rect.options.depth_offset);
            self.selection = picked.map(|(image, start)| Selection {
                entity: image.ent_path.clone(),
                resolution: image.resolution(),
                start,
                region: None,
            });
            self.dragging = self.selection.is_some();
        }
        if self.dragging {
            if let Some(selection) = &mut self.selection
                && let Some(image) = images().find(|image| {
                    image.ent_path == selection.entity && image.resolution() == selection.resolution
                })
                && let Some(plane) = ImagePlane::from_image(image)
                && let Some(end_ui) = response.interact_pointer_pos()
            {
                selection.region = PixelRegion::from_drag(
                    selection.start,
                    plane.image_from_ui(end_ui, ui_from_scene),
                    selection.resolution,
                );
            }
            if response.drag_stopped_by(egui::PointerButton::Primary)
                || !response.ctx.input(|i| i.pointer.primary_down())
            {
                self.dragging = false;
            }
        }
    }

    pub fn paint(
        &mut self,
        ctx: &ViewerContext<'_>,
        ui: &egui::Ui,
        view_id: re_viewer_context::ViewId,
        ui_from_scene: RectTransform,
        output: &SystemExecutionOutput,
    ) {
        let Some(selection) = &self.selection else {
            return;
        };
        let Some(region) = selection.region else {
            return;
        };
        let Some(image) = iter_spatial_data(output)
            .flat_map(|data| &data.pickable_rects)
            .find(|image| selection.matches(image))
        else {
            return;
        };
        let Some(plane) = ImagePlane::from_image(image) else {
            return;
        };
        let bounds = region.rect();
        let points = [
            bounds.left_top(),
            bounds.right_top(),
            bounds.right_bottom(),
            bounds.left_bottom(),
        ]
        .map(|point| plane.ui_from_image(point, ui_from_scene));
        ui.painter().add(egui::Shape::closed_line(
            points.to_vec(),
            (3.0, ui.visuals().extreme_bg_color),
        ));
        ui.painter().add(egui::Shape::closed_line(
            points.to_vec(),
            ui.visuals().selection.stroke,
        ));

        let mut clear = false;
        egui::Window::new("Image region")
            .id(ui.make_persistent_id(("image_region", view_id)))
            .default_pos(ui.max_rect().right_top() + egui::vec2(-320.0, 12.0))
            .default_width(280.0)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label(selection.entity.to_string());
                let bounds = region.rect();
                ui.label(format!(
                    "{} × {} pixels",
                    re_format::format_uint(region.max[0] - region.min[0]),
                    re_format::format_uint(region.max[1] - region.min[1])
                ));
                ui.label(format!(
                    "x [{}, {}), y [{}, {})",
                    region.min[0], region.max[0], region.min[1], region.max[1]
                ));
                // Match the existing pixel hover magnifier's five UI points per texel.
                let scale = 5.0;
                ui.label("5× magnification — scroll to inspect large regions");
                egui::ScrollArea::both()
                    .max_width(280.0)
                    .max_height(220.0)
                    .show(ui, |ui| {
                        let (preview, _) =
                            ui.allocate_exact_size(bounds.size() * scale, egui::Sense::hover());
                        let full_image = Rect::from_min_size(
                            preview.min - bounds.min.to_vec2() * scale,
                            Vec2::new(
                                selection.resolution[0] as f32,
                                selection.resolution[1] as f32,
                            ) * scale,
                        );
                        if let Err(err) = gpu_bridge::render_image(
                            ctx.render_ctx(),
                            &ui.painter().with_clip_rect(preview),
                            full_image,
                            image.textured_rect.colormapped_texture.clone(),
                            egui::TextureOptions::NEAREST,
                            re_renderer::ViewBuilderId::new(
                                re_log_types::hash::Hash64::hash((view_id, "image_region"))
                                    .hash64(),
                            ),
                            "image region".into(),
                        ) {
                            ui.error_with_details_on_hover(err.to_string());
                        }
                    });
                clear = ui.button("Clear region").clicked();
            });
        if clear {
            self.selection = None;
            self.dragging = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_rounds_outward_and_reverse_drag_is_identical() {
        let start = egui::pos2(2.2, 3.8);
        let end = egui::pos2(8.1, 9.0);
        let expected = Some(PixelRegion {
            min: [2, 3],
            max: [9, 9],
        });
        assert_eq!(PixelRegion::from_drag(start, end, [10, 10]), expected);
        assert_eq!(PixelRegion::from_drag(end, start, [10, 10]), expected);
    }

    #[test]
    fn region_clamps_to_image_and_rejects_empty_or_invalid_input() {
        assert_eq!(
            PixelRegion::from_drag(egui::pos2(2.0, 3.0), egui::pos2(20.0, -5.0), [10, 8]),
            Some(PixelRegion {
                min: [2, 0],
                max: [10, 3]
            })
        );
        for (start, end) in [
            (egui::pos2(-10.0, -10.0), egui::pos2(-1.0, -1.0)),
            (egui::pos2(2.0, 2.0), egui::pos2(2.0, 5.0)),
            (egui::pos2(f32::NAN, 2.0), egui::pos2(5.0, 5.0)),
            (egui::pos2(1.0, 2.0), egui::pos2(f32::INFINITY, 5.0)),
        ] {
            assert_eq!(PixelRegion::from_drag(start, end, [10, 8]), None);
        }
        assert_eq!(
            PixelRegion::from_drag(Pos2::ZERO, egui::pos2(5.0, 5.0), [0, 8]),
            None
        );
    }

    #[test]
    fn transformed_image_coordinates_survive_pan_and_zoom() {
        let plane = ImagePlane::new(
            glam::vec2(30.0, -15.0),
            glam::vec2(0.0, 2.0),
            glam::vec2(-3.0, 0.5),
        )
        .unwrap();
        let pixel = egui::pos2(13.25, 27.5);
        for viewport in [
            Rect::from_min_max(Pos2::ZERO, egui::pos2(500.0, 400.0)),
            Rect::from_min_max(egui::pos2(-120.0, 35.0), egui::pos2(880.0, 835.0)),
        ] {
            let transform = RectTransform::from_to(
                Rect::from_min_max(egui::pos2(-100.0, -100.0), egui::pos2(100.0, 100.0)),
                viewport,
            );
            let actual = plane.image_from_ui(plane.ui_from_image(pixel, transform), transform);
            assert!(
                (actual - pixel).length() < 0.0001,
                "round-trip changed image coordinates: {actual:?}"
            );
        }
        assert!(ImagePlane::new(glam::Vec2::ZERO, glam::Vec2::X, glam::Vec2::X).is_none());
        assert!(
            ImagePlane::new(
                glam::Vec2::ZERO,
                glam::Vec2::X * f32::MAX,
                glam::Vec2::Y * f32::MAX
            )
            .is_none()
        );
        assert!(ImagePlane::new(glam::Vec2::NAN, glam::Vec2::X, glam::Vec2::Y).is_none());
    }
}
