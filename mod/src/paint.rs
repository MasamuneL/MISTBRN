//! Drawing in a local space that can be scaled (P7b: the book and HUD scale settings).
//!
//! At 100 % every call goes straight to the `DrawContext`, offset by the origin. At any other
//! scale the calls are recorded as `GuiDraw`s and painted in one `gui_paint` under a pose
//! translated to the origin and scaled, so text scales with the rest. A clip flushes what was
//! recorded, because `push_clip` takes effect at once and in screen pixels.
use mod_api::client::hud_layout::{Blend, PoseOp};
use mod_api::{DrawContext, GuiDraw, Text};
use std::borrow::Cow;

pub(crate) struct Painter<'a> {
    draw: &'a mut dyn DrawContext,
    origin: [f32; 2],
    scale: f32,
    recorded: Vec<GuiDraw>,
    clips: u32,
}

impl<'a> Painter<'a> {
    pub(crate) fn new(draw: &'a mut dyn DrawContext, origin: [f32; 2], scale: f32) -> Self {
        Self {
            draw,
            origin,
            scale,
            recorded: Vec::new(),
            clips: 0,
        }
    }

    fn direct(&self) -> bool {
        self.scale == 1.0
    }

    fn at(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        [self.origin[0] + x, self.origin[1] + y]
    }

    /// A local point to screen GUI pixels.
    pub(crate) fn to_screen(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        [
            self.origin[0] + x * self.scale,
            self.origin[1] + y * self.scale,
        ]
    }

    pub(crate) fn text_width(&self, text: &str) -> f32 {
        self.draw.text_width(text)
    }

    pub(crate) fn fill(&mut self, [x, y, w, h]: [f32; 4], argb: u32) {
        if self.direct() {
            let [x, y] = self.at([x, y]);
            self.draw.fill([x, y, w, h], argb);
            return;
        }
        let (x0, y0) = (x.round() as i32, y.round() as i32);
        self.recorded.push(GuiDraw::Fill {
            x0,
            y0,
            x1: x0 + w.round() as i32,
            y1: y0 + h.round() as i32,
            color: argb,
        });
    }

    /// Part `src` of `sheet` (a `mistbrn:gui/...` texture `size` pixels big) drawn at `dst`.
    /// False when the direct path found no such sprite (the recorded path cannot tell, and
    /// returns true).
    pub(crate) fn sprite(
        &mut self,
        sheet: &'static str,
        size: [f32; 2],
        src: [f32; 4],
        dst: [f32; 4],
        tint: Option<u32>,
    ) -> bool {
        if self.direct() {
            let [x, y] = self.at([dst[0], dst[1]]);
            return self.draw.sprite(sheet, src, [x, y, dst[2], dst[3]], tint);
        }
        self.recorded.push(GuiDraw::Blit {
            texture: Cow::Borrowed(sheet),
            blend: Blend::Textured,
            x: dst[0].round() as i32,
            y: dst[1].round() as i32,
            w: dst[2].round() as i32,
            h: dst[3].round() as i32,
            uv: [src[0], src[1]],
            src: [src[2].round() as i32, src[3].round() as i32],
            tex: [size[0] as i32, size[1] as i32],
            color: tint.unwrap_or(0xFFFF_FFFF),
        });
        true
    }

    pub(crate) fn text(&mut self, text: &str, at: [f32; 2], argb: u32, shadow: bool) {
        if self.direct() {
            let at = self.at(at);
            self.draw.text(text, at, argb, shadow);
            return;
        }
        self.recorded.push(GuiDraw::Text {
            text: Text::literal(text),
            x: at[0].round() as i32,
            y: at[1].round() as i32,
            color: argb,
            shadow,
        });
    }

    /// Cuts what is drawn next to the local `rect`, until `pop_clip`.
    pub(crate) fn push_clip(&mut self, [x, y, w, h]: [f32; 4]) {
        self.flush();
        let [sx, sy] = self.to_screen([x, y]);
        if self
            .draw
            .push_clip([sx, sy, w * self.scale, h * self.scale])
            .is_ok()
        {
            self.clips += 1;
        }
    }

    pub(crate) fn pop_clip(&mut self) {
        self.flush();
        if self.clips > 0 {
            // Counted down even if refused, so `drop` never loops on a clip it cannot close.
            self.clips -= 1;
            let _ = self.draw.pop_clip();
        }
    }

    /// A vanilla tooltip at the local point `at` (drawn at once, over what came before).
    pub(crate) fn tooltip(&mut self, lines: &[String], at: [f32; 2]) {
        self.flush();
        let lines: Vec<Text> = lines.iter().map(|l| Text::literal(l.clone())).collect();
        let at = self.to_screen(at);
        if let Err(error) = self.draw.gui_tooltip(&lines, at) {
            eprintln!("MISTBRN event=tooltip result=refused:{error:?}");
        }
    }

    /// Paints what was recorded, under the origin and scale.
    pub(crate) fn flush(&mut self) {
        if self.recorded.is_empty() {
            return;
        }
        let mut draws = Vec::with_capacity(self.recorded.len() + 2);
        draws.push(GuiDraw::Push(vec![
            PoseOp::Translate(self.origin[0], self.origin[1]),
            PoseOp::Scale(self.scale, self.scale),
        ]));
        draws.append(&mut self.recorded);
        draws.push(GuiDraw::Pop);
        if let Err(error) = self.draw.gui_paint(&draws) {
            eprintln!("MISTBRN event=paint result=refused:{error:?}");
        }
    }
}

impl Drop for Painter<'_> {
    fn drop(&mut self) {
        self.flush();
        while self.clips > 0 {
            self.pop_clip();
        }
    }
}
