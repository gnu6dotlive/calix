use crate::icons::AppIcon;
use gpui::*;
use std::time::{Duration, Instant};

pub const TOAST_DISPLAY_DURATION: Duration = Duration::from_millis(3500);
pub const TOAST_ANIMATION_DURATION: Duration = Duration::from_millis(120);
pub const TOAST_MAX_VISIBLE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    pub fn icon(&self) -> AppIcon {
        match self {
            ToastKind::Info => AppIcon::Info,
            ToastKind::Success => AppIcon::Check,
            ToastKind::Warning => AppIcon::Warning,
            ToastKind::Error => AppIcon::Close,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastAction {
    OpenLogs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLifecycle {
    Entering,
    Visible,
    Exiting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub title: String,
    pub message: String,
    pub action: Option<ToastAction>,
    pub lifecycle: ToastLifecycle,
    pub created_at: Instant,
}

pub struct ToastCenter {
    pub next_id: u64,
    pub toasts: Vec<Toast>,
}

impl ToastCenter {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            toasts: Vec::new(),
        }
    }

    /// Ajout pur d'un toast avec éviction déterministe préservant les erreurs
    pub fn push_toast(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
        action: Option<ToastAction>,
        now: Instant,
        reduce_motion: bool,
    ) -> u64 {
        self.next_id += 1;
        let id = self.next_id;

        if self.toasts.len() >= TOAST_MAX_VISIBLE {
            // Éviction prioritaire du plus ancien toast qui n'est PAS une erreur
            let evict_idx = self
                .toasts
                .iter()
                .position(|t| t.kind != ToastKind::Error)
                .unwrap_or(0);
            self.toasts.remove(evict_idx);
        }

        let lifecycle = if reduce_motion {
            ToastLifecycle::Visible
        } else {
            ToastLifecycle::Entering
        };

        self.toasts.push(Toast {
            id,
            kind,
            title: title.into(),
            message: message.into(),
            action,
            lifecycle,
            created_at: now,
        });

        id
    }

    /// Vérifie si une transition de cycle de vie est autorisée.
    /// Les transitions autorisées sont strictement :
    /// - `Entering -> Visible`
    /// - `Entering -> Exiting`
    /// - `Visible  -> Exiting`
    ///
    /// Toute régression (ex. `Exiting -> Visible`) est formellement interdite.
    pub fn is_valid_transition(from: ToastLifecycle, to: ToastLifecycle) -> bool {
        matches!(
            (from, to),
            (ToastLifecycle::Entering, ToastLifecycle::Visible)
                | (ToastLifecycle::Entering, ToastLifecycle::Exiting)
                | (ToastLifecycle::Visible, ToastLifecycle::Exiting)
        )
    }

    /// Transitionne le cycle de vie de manière conditionnelle si l'état actuel correspond exactement à `expected`
    /// et si la transition est formellement valide.
    /// Garantit qu'un toast marqué `Exiting` ne régresse jamais vers `Visible`.
    pub fn transition_lifecycle(
        &mut self,
        id: u64,
        expected: ToastLifecycle,
        next: ToastLifecycle,
    ) -> bool {
        if !Self::is_valid_transition(expected, next) {
            return false;
        }
        if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
            if toast.lifecycle == expected {
                toast.lifecycle = next;
                return true;
            }
        }
        false
    }

    pub fn remove_toast(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Ferme un toast avec animation de sortie (120ms) ou suppression immédiate sous reduce_motion
    pub fn dismiss(&mut self, id: u64, reduce_motion: bool, cx: &mut Context<Self>) {
        if reduce_motion {
            self.remove_toast(id);
            cx.notify();
        } else if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
            if toast.lifecycle != ToastLifecycle::Exiting {
                toast.lifecycle = ToastLifecycle::Exiting;
                cx.notify();

                cx.spawn(async move |this, cx| {
                    cx.background_executor()
                        .timer(TOAST_ANIMATION_DURATION)
                        .await;
                    let _ = this.update(cx, |center, cx| {
                        center.remove_toast(id);
                        cx.notify();
                    });
                })
                .detach();
            }
        }
    }

    /// Poste un toast dans l'entité et planifie son cycle de vie via l'exécuteur GPUI
    pub fn post(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
        action: Option<ToastAction>,
        reduce_motion: bool,
        cx: &mut Context<Self>,
    ) -> u64 {
        let id = self.push_toast(kind, title, message, action, Instant::now(), reduce_motion);
        cx.notify();

        if reduce_motion {
            // Sous reduced_motion : pas d'animation d'entrée ou de sortie, disparition directe après 3.5s
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(TOAST_DISPLAY_DURATION).await;
                let _ = this.update(cx, |center, cx| {
                    center.remove_toast(id);
                    cx.notify();
                });
            })
            .detach();
        } else {
            // Transition d'entrée -> Visible (120ms)
            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(TOAST_ANIMATION_DURATION)
                    .await;
                let transitioned = this
                    .update(cx, |center, cx| {
                        let ok = center.transition_lifecycle(
                            id,
                            ToastLifecycle::Entering,
                            ToastLifecycle::Visible,
                        );
                        if ok {
                            cx.notify();
                        }
                        ok
                    })
                    .unwrap_or(false);

                // Si le toast n'est plus en Entering (ex. déjà Exiting suite à dismiss ou supprimé), abandonner la suite
                if !transitioned {
                    return;
                }

                // Durée d'affichage (3.5s)
                cx.background_executor().timer(TOAST_DISPLAY_DURATION).await;
                let transitioned = this
                    .update(cx, |center, cx| {
                        let ok = center.transition_lifecycle(
                            id,
                            ToastLifecycle::Visible,
                            ToastLifecycle::Exiting,
                        );
                        if ok {
                            cx.notify();
                        }
                        ok
                    })
                    .unwrap_or(false);

                if !transitioned {
                    return;
                }

                // Transition de sortie (120ms) puis suppression
                cx.background_executor()
                    .timer(TOAST_ANIMATION_DURATION)
                    .await;
                let _ = this.update(cx, |center, cx| {
                    center.remove_toast(id);
                    cx.notify();
                });
            })
            .detach();
        }

        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_toast_monotonic_ids() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id1 = center.push_toast(ToastKind::Info, "T1", "M1", None, now, false);
        let id2 = center.push_toast(ToastKind::Success, "T2", "M2", None, now, false);
        let id3 = center.push_toast(ToastKind::Error, "T3", "M3", None, now, false);

        assert!(id2 > id1);
        assert!(id3 > id2);
        assert_eq!(center.toasts.len(), 3);
    }

    #[test]
    fn test_toast_max_visible_bound() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        center.push_toast(ToastKind::Info, "T1", "M1", None, now, false);
        center.push_toast(ToastKind::Info, "T2", "M2", None, now, false);
        center.push_toast(ToastKind::Info, "T3", "M3", None, now, false);
        assert_eq!(center.toasts.len(), TOAST_MAX_VISIBLE);

        // Fourth toast must evict oldest
        let id4 = center.push_toast(ToastKind::Info, "T4", "M4", None, now, false);
        assert_eq!(center.toasts.len(), TOAST_MAX_VISIBLE);
        assert_eq!(center.toasts.last().unwrap().id, id4);
        assert!(!center.toasts.iter().any(|t| t.title == "T1"));
    }

    #[test]
    fn test_toast_deterministic_eviction_preserves_error() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        // Insert Error first, then Info, then Success
        let err_id = center.push_toast(ToastKind::Error, "Fatal Error", "E1", None, now, false);
        center.push_toast(ToastKind::Info, "Notice", "I1", None, now, false);
        center.push_toast(ToastKind::Success, "Done", "S1", None, now, false);
        assert_eq!(center.toasts.len(), 3);

        // Fourth insertion should evict the oldest NON-error toast (which is "Notice"), preserving "Fatal Error"
        center.push_toast(ToastKind::Info, "New Notice", "I2", None, now, false);
        assert_eq!(center.toasts.len(), 3);
        assert!(
            center.toasts.iter().any(|t| t.id == err_id),
            "Deterministic eviction must preserve error toast when non-errors are available"
        );
        assert!(
            !center.toasts.iter().any(|t| t.title == "Notice"),
            "Oldest non-error toast should have been evicted"
        );
    }

    #[test]
    fn test_toast_lifecycle_transitions() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id = center.push_toast(ToastKind::Info, "T", "M", None, now, false);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Entering);

        let ok1 =
            center.transition_lifecycle(id, ToastLifecycle::Entering, ToastLifecycle::Visible);
        assert!(ok1);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Visible);

        let ok2 = center.transition_lifecycle(id, ToastLifecycle::Visible, ToastLifecycle::Exiting);
        assert!(ok2);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Exiting);

        center.remove_toast(id);
        assert!(center.toasts.is_empty());
    }

    #[test]
    fn test_toast_timer_target_identity() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id1 = center.push_toast(ToastKind::Info, "T1", "M1", None, now, false);
        let id2 = center.push_toast(ToastKind::Info, "T2", "M2", None, now, false);

        // Dismissing id1 does not affect id2
        center.remove_toast(id1);
        assert_eq!(center.toasts.len(), 1);
        assert_eq!(center.toasts[0].id, id2);
    }

    #[test]
    fn test_toast_reduced_motion_immediate() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id = center.push_toast(ToastKind::Success, "T", "M", None, now, true);
        assert_eq!(
            center.toasts[0].lifecycle,
            ToastLifecycle::Visible,
            "Under reduced motion, toast lifecycle must immediately snap to Visible"
        );
        assert_eq!(center.toasts[0].id, id);
    }

    #[test]
    fn test_toast_dismiss_transitions() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id = center.push_toast(ToastKind::Info, "T", "M", None, now, false);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Entering);
        let ok = center.transition_lifecycle(id, ToastLifecycle::Entering, ToastLifecycle::Exiting);
        assert!(ok);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Exiting);
    }

    #[test]
    fn test_toast_dismiss_during_entering_prevents_visible_regression() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id = center.push_toast(ToastKind::Info, "T", "M", None, now, false);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Entering);

        // L'utilisateur clique sur fermer (dismiss) pendant les 120ms d'Entering
        let ok = center.transition_lifecycle(id, ToastLifecycle::Entering, ToastLifecycle::Exiting);
        assert!(ok);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Exiting);

        // Le timer d'entrée réveillé après 120ms tente de passer Entering -> Visible
        let transitioned =
            center.transition_lifecycle(id, ToastLifecycle::Entering, ToastLifecycle::Visible);
        // La régression vers Visible doit être strictement rejetée
        assert!(!transitioned);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Exiting);

        // Vérification des transitions interdites : Exiting ne peut pas devenir Visible
        let invalid_transition =
            center.transition_lifecycle(id, ToastLifecycle::Exiting, ToastLifecycle::Visible);
        assert!(!invalid_transition);
    }

    #[test]
    fn test_toast_deterministic_eviction_all_errors_evicts_oldest() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let err1 = center.push_toast(ToastKind::Error, "E1", "M1", None, now, false);
        let err2 = center.push_toast(ToastKind::Error, "E2", "M2", None, now, false);
        let err3 = center.push_toast(ToastKind::Error, "E3", "M3", None, now, false);
        assert_eq!(center.toasts.len(), 3);

        // Si une 4ème erreur arrive alors que la file n'a que des erreurs,
        // la file bornée doit nécessairement évincer la plus ancienne (err1)
        let err4 = center.push_toast(ToastKind::Error, "E4", "M4", None, now, false);
        assert_eq!(center.toasts.len(), 3);
        assert_eq!(center.toasts[0].id, err2);
        assert_eq!(center.toasts[1].id, err3);
        assert_eq!(center.toasts[2].id, err4);
        assert!(!center.toasts.iter().any(|t| t.id == err1));
    }
}
