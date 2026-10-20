use crate::components::log_drawer::{LogDrawer, LogDrawerProps};
use crate::state::console::{ConsoleEvent, ConsoleModel};
use crate::state::toast::{ToastCenter, ToastKind};
use crate::state::{AnimatedScalar, MotionDurations};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;
use std::time::Instant;

/// Vue d'état isolée pour le tiroir de console d'opérations.
/// Possède son propre cycle d'animation continue (AnimatedScalar),
/// évitant toute propagation de frames au niveau du Workspace racine lors de l'ouverture/fermeture.
pub struct OperationConsoleView {
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    pub height_scalar: AnimatedScalar,
    pub configured_height: f32,
    pub theme: Theme,
    pub reduce_motion: bool,
    pub logs_copied_feedback: bool,
    _console_sub: Subscription,
}

impl OperationConsoleView {
    pub fn new(
        console: Entity<ConsoleModel>,
        toast_center: Entity<ToastCenter>,
        configured_height: f32,
        theme: Theme,
        reduce_motion: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let is_open = console.read(cx).is_open;
        let initial_height = if is_open { configured_height } else { 0.0 };
        let mut height_scalar = AnimatedScalar::new(initial_height);
        height_scalar.easing = crate::state::motion::ease_in_out;

        let console_sub = cx.subscribe(&console, |this, _console, event, cx| match event {
            ConsoleEvent::Toggled(open) => {
                let target = if *open { this.configured_height } else { 0.0 };
                let duration = if *open
                    && matches!(
                        this.console.read(cx).status,
                        crate::state::OperationStatus::Error(_)
                    ) {
                    MotionDurations::EMPHASIS
                } else {
                    MotionDurations::STANDARD
                };
                this.height_scalar
                    .retarget(target, duration, Instant::now(), this.reduce_motion);
                cx.notify();
            }
            ConsoleEvent::OperationStarted(_) => {
                cx.notify();
            }
            ConsoleEvent::OperationFinished(_) => {
                cx.notify();
            }
            _ => {
                cx.notify();
            }
        });

        Self {
            console,
            toast_center,
            height_scalar,
            configured_height,
            theme,
            reduce_motion,
            logs_copied_feedback: false,
            _console_sub: console_sub,
        }
    }

    pub fn set_reduce_motion(&mut self, reduce_motion: bool, cx: &mut Context<Self>) {
        self.reduce_motion = reduce_motion;
        if reduce_motion {
            self.height_scalar.snap(self.height_scalar.target_value);
        }
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        cx.notify();
    }

    pub fn set_configured_height(&mut self, height: f32, cx: &mut Context<Self>) {
        self.configured_height = height;
        if self.console.read(cx).is_open {
            self.height_scalar.snap(height);
        }
        cx.notify();
    }

    pub fn copy_logs(&mut self, cx: &mut Context<Self>) {
        let logs_text = self.console.read(cx).copy_text();

        cx.write_to_clipboard(ClipboardItem::new_string(logs_text));
        self.logs_copied_feedback = true;
        let reduce = self.reduce_motion;
        self.toast_center.update(cx, |tc, cx| {
            tc.post(
                ToastKind::Info,
                "Logs copied",
                "Console output has been copied to clipboard.",
                None,
                reduce,
                cx,
            );
        });
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;
            let _ = this.update(cx, |view, cx| {
                view.logs_copied_feedback = false;
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for OperationConsoleView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let animating = self.height_scalar.update(Instant::now());
        if animating || self.height_scalar.is_active() {
            window.request_animation_frame();
        }

        let current_height = self.height_scalar.current;
        let (is_open, auto_scroll) = {
            let c = self.console.read(cx);
            (c.is_open, c.auto_scroll)
        };

        let entity = cx.entity().clone();
        let entity_toggle = entity.clone();
        let entity_copy = entity.clone();
        let entity_clear = entity.clone();
        let entity_auto = entity.clone();
        let entity_raw = entity.clone();

        let console_read = self.console.read(cx);

        LogDrawer::render(LogDrawerProps {
            logs: &console_read.logs,
            backend_phases: &console_read.backend_phases,
            status: &console_read.status,
            is_open,
            auto_scroll,
            show_raw_logs: console_read.show_raw_logs,
            height: current_height,
            copied_feedback: self.logs_copied_feedback,
            scroll_handle: &console_read.scroll_handle,
            theme: &self.theme,
            on_toggle: Some(Rc::new(move |_w, cx| {
                entity_toggle.update(cx, |this, cx| {
                    this.console.update(cx, |c, cx| c.toggle_drawer(cx));
                });
            })),
            on_copy: Some(Rc::new(move |_w, cx| {
                entity_copy.update(cx, |this, cx| {
                    this.copy_logs(cx);
                });
            })),
            on_clear: Some(Rc::new(move |_w, cx| {
                entity_clear.update(cx, |this, cx| {
                    this.console.update(cx, |c, cx| c.clear_logs(cx));
                });
            })),
            on_toggle_autoscroll: Some(Rc::new(move |_w, cx| {
                entity_auto.update(cx, |this, cx| {
                    this.console.update(cx, |c, cx| c.toggle_auto_scroll(cx));
                });
            })),
            on_toggle_raw_logs: Some(Rc::new(move |_w, cx| {
                entity_raw.update(cx, |this, cx| {
                    this.console.update(cx, |c, cx| c.toggle_raw_logs(cx));
                });
            })),
        })
    }
}
