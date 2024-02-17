use egui::Image;
use egui::Pos2;
use egui::Rect;
use egui::Response;
use egui::Sense;
use egui::Ui;
use egui::Vec2;
use egui::Widget;

pub struct ImageProgressBar<'a> {
    left_frame_image: Image<'a>,
    center_frame_image: Image<'a>,
    right_frame_image: Image<'a>,
    fill_image: Image<'a>,

    progress: f32,
    desired_width: Option<f32>,
}

impl<'a> ImageProgressBar<'a> {
    pub fn new(
        left_frame_image: impl Into<Image<'a>>,
        center_frame_image: impl Into<Image<'a>>,
        right_frame_image: impl Into<Image<'a>>,
        fill_image: impl Into<Image<'a>>,
    ) -> Self {
        Self {
            left_frame_image: left_frame_image.into(),
            center_frame_image: center_frame_image.into(),
            right_frame_image: right_frame_image.into(),
            fill_image: fill_image.into(),

            progress: 0.0,
            desired_width: None,
        }
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }
}

impl<'a> Widget for ImageProgressBar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let left_frame_size = self.left_frame_image.size().unwrap_or(Vec2::ZERO);
        let center_frame_size = self.center_frame_image.size().unwrap_or(Vec2::ZERO);
        let right_frame_size = self.right_frame_image.size().unwrap_or(Vec2::ZERO);
        let fill_size = self.fill_image.size().unwrap_or(Vec2::ZERO);

        let width = self.desired_width.unwrap_or_else(|| ui.available_size_before_wrap().x);
        let height = left_frame_size.y.max(center_frame_size.y).max(right_frame_size.y);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());

        if ui.is_rect_visible(rect) {
            let left_frame_rect = Rect::from_min_max(
                Pos2::new(rect.min.x, rect.min.y + (rect.height() - left_frame_size.y) / 2.0),
                Pos2::new(rect.min.x + left_frame_size.x, rect.max.y - (rect.height() - left_frame_size.y) / 2.0),
            );

            let center_frame_rect = Rect::from_min_max(
                Pos2::new(rect.min.x + left_frame_size.x, rect.min.y + (rect.height() - center_frame_size.y) / 2.0),
                Pos2::new(rect.max.x - right_frame_size.x, rect.max.y - (rect.height() - center_frame_size.y) / 2.0),
            );

            let right_frame_rect = Rect::from_min_max(
                Pos2::new(rect.max.x - right_frame_size.x, rect.min.y + (rect.height() - right_frame_size.y) / 2.0),
                Pos2::new(rect.max.x, rect.max.y - (rect.height() - right_frame_size.y) / 2.0),
            );

            let mut fill_rect = Rect::from_min_max(
                Pos2::new(rect.min.x + left_frame_size.x, rect.min.y + (rect.height() - fill_size.y) / 2.0),
                Pos2::new(rect.max.x - right_frame_size.x, rect.max.y - (rect.height() - fill_size.y) / 2.0),
            );
            fill_rect.max.x -= fill_rect.width() * (1.0 - self.progress);

            self.left_frame_image.paint_at(ui, left_frame_rect);
            self.center_frame_image.paint_at(ui, center_frame_rect);
            self.right_frame_image.paint_at(ui, right_frame_rect);
            self.fill_image.paint_at(ui, fill_rect);
        }
        response
    }
}
