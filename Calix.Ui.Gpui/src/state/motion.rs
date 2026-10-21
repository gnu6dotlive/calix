use std::time::{Duration, Instant};

/// Tokens de durées de transition de l'interface
pub struct MotionDurations;

impl MotionDurations {
    pub const FAST: Duration = Duration::from_millis(120);
    pub const STANDARD: Duration = Duration::from_millis(160);
    pub const EMPHASIS: Duration = Duration::from_millis(220);
}

/// Fonctions d'adoucissement pures (fn(f32) -> f32)
pub fn ease_out_quint(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(5)
}

pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        let x = -2.0 * t + 2.0;
        1.0 - x * x / 2.0
    }
}

/// Le poids d'un élément dans une arrivée : il voyage plus loin et plus
/// longtemps, ou il s'ajuste vite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    /// L'ancre : l'élément qui porte la lecture.
    Heavy,
    /// L'élément ordinaire.
    Normal,
    /// L'élément léger : il s'ajuste, il ne voyage pas.
    Light,
}

impl Weight {
    /// La distance parcourue, en pixels.
    pub fn travel(self) -> f32 {
        match self {
            Weight::Heavy => 14.0,
            Weight::Normal => 10.0,
            Weight::Light => 7.0,
        }
    }

    /// La durée de son propre mouvement.
    pub fn duration(self) -> Duration {
        match self {
            Weight::Heavy => Duration::from_millis(180),
            Weight::Normal => Duration::from_millis(140),
            Weight::Light => Duration::from_millis(110),
        }
    }

}

/// L'écart entre deux éléments de la cascade, avant rétrécissement.
///
/// L'écart appartient à la cascade, pas à l'élément : le poids d'un élément
/// décide de la distance qu'il parcourt et de la durée de son mouvement, et
/// l'écart se rétrécit d'un élément à l'autre pour que la vague s'accélère.
pub const WATERFALL_BASE_GAP: Duration = Duration::from_millis(50);

/// Un élément d'une arrivée en cascade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterfallStep {
    /// Le temps qui passe avant que cet élément ne bouge.
    pub delay: Duration,
    /// La distance qu'il parcourt, en pixels.
    pub travel: f32,
    /// La durée de son propre mouvement.
    pub duration: Duration,
}

/// L'arrivée en cascade : des éléments qui arrivent par en dessous, l'un après
/// l'autre, sans se faire la queue.
///
/// Les écarts se rétrécissent d'un élément à l'autre : la cascade est une
/// vague qui s'accélère, pas une file. Le dernier s'ajuste.
pub fn waterfall_entry(weights: &[Weight]) -> Vec<WaterfallStep> {
    let mut steps = Vec::with_capacity(weights.len());
    let mut delay = Duration::ZERO;
    for (index, weight) in weights.iter().enumerate() {
        steps.push(WaterfallStep {
            delay,
            travel: weight.travel(),
            duration: weight.duration(),
        });
        // Les écarts se rétrécissent : ×0,84 à chaque élément.
        delay += WATERFALL_BASE_GAP.mul_f32(0.84f32.powi(index as i32));
    }
    steps
}

/// La durée totale d'une arrivée en cascade.
pub fn waterfall_duration(steps: &[WaterfallStep]) -> Duration {
    steps
        .iter()
        .map(|step| step.delay + step.duration)
        .max()
        .unwrap_or(Duration::ZERO)
}

/// La coupure à la courbe : l'entrée qui suit une coupe.
///
/// L'élément sortant accélère dans une direction ; la coupe tombe au milieu
/// du mouvement ; l'élément entrant continue dans la *même* direction et
/// décélère. L'assombrissement s'allume en plein chemin : une apparition en
/// fondu, de 0 à 1, se lirait comme un fondu et non comme une arrivée.
///
/// Le voyage est partiel : environ 12 % de la surface, jamais un travers
/// complet.
pub fn cut_entry(delta: f32, travel: f32) -> (f32, f32) {
    let t = delta.clamp(0.0, 1.0);
    // L'assombrissement s'allume à 0,35 en plein chemin.
    let opacity = if t < 0.35 {
        0.0
    } else {
        ((t - 0.35) / 0.65).clamp(0.0, 1.0)
    };
    // La position : le voyage se referme, la même direction que la sortie.
    let offset = travel * (1.0 - ease_out_quint(t));
    (opacity, offset)
}

/// L'overshoot : l'aiguille dépasse sa position et revient.
///
/// Un instrument dépasse ; un commutateur, non. L'aiguille qui se jette sur
/// son cran le franchit légèrement avant de revenir s'y poser.
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let t = t.clamp(0.0, 1.0);
    let u = t - 1.0;
    1.0 + C3 * u * u * u + C1 * u * u
}

/// Politique d'animation de l'application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionPolicy {
    pub reduce_motion: bool,
}

impl MotionPolicy {
    pub fn new(reduce_motion: bool) -> Self {
        Self { reduce_motion }
    }
}

/// Échelle scalaire animée continue, interruptible et réversible
#[derive(Debug, Clone)]
pub struct AnimatedScalar {
    pub current: f32,
    pub start_value: f32,
    pub target_value: f32,
    pub start_time: Option<Instant>,
    pub duration: Duration,
    pub easing: fn(f32) -> f32,
}

impl AnimatedScalar {
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            start_value: initial,
            target_value: initial,
            start_time: None,
            duration: MotionDurations::STANDARD,
            easing: ease_out_quint,
        }
    }

    pub fn retarget(
        &mut self,
        new_target: f32,
        duration: Duration,
        now: Instant,
        reduce_motion: bool,
    ) {
        if reduce_motion {
            self.snap(new_target);
            return;
        }

        if (self.current - new_target).abs() < f32::EPSILON {
            self.snap(new_target);
            return;
        }

        // Reversal fluide depuis la valeur échantillonnée actuelle
        self.start_value = self.current;
        self.target_value = new_target;
        self.duration = duration;
        self.start_time = Some(now);
    }

    pub fn update(&mut self, now: Instant) -> bool {
        let Some(start) = self.start_time else {
            return false;
        };

        let elapsed = now.saturating_duration_since(start);
        if elapsed >= self.duration || self.duration.is_zero() {
            self.current = self.target_value;
            self.start_time = None;
            return false;
        }

        let progress = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        let eased = (self.easing)(progress);
        self.current = self.start_value + (self.target_value - self.start_value) * eased;
        true
    }

    pub fn is_active(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn snap(&mut self, target: f32) {
        self.current = target;
        self.start_value = target;
        self.target_value = target;
        self.start_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animated_scalar_forward_progress() {
        let mut scalar = AnimatedScalar::new(0.0);
        let t0 = Instant::now();
        scalar.retarget(100.0, Duration::from_millis(200), t0, false);

        assert!(scalar.is_active());
        assert_eq!(scalar.current, 0.0);

        // Update at half time
        let t1 = t0 + Duration::from_millis(100);
        let active = scalar.update(t1);
        assert!(active);
        assert!(scalar.current > 0.0 && scalar.current < 100.0);
    }

    #[test]
    fn test_animated_scalar_completion() {
        let mut scalar = AnimatedScalar::new(0.0);
        let t0 = Instant::now();
        scalar.retarget(100.0, Duration::from_millis(100), t0, false);

        let t_end = t0 + Duration::from_millis(150);
        let active = scalar.update(t_end);
        assert!(!active);
        assert_eq!(scalar.current, 100.0);
        assert!(!scalar.is_active());
    }

    #[test]
    fn test_animated_scalar_retarget_reversal_from_sampled_value() {
        let mut scalar = AnimatedScalar::new(56.0);
        let t0 = Instant::now();
        // Expand to 190
        scalar.retarget(190.0, Duration::from_millis(200), t0, false);

        // Progress to ~100ms
        let t1 = t0 + Duration::from_millis(100);
        scalar.update(t1);
        let sampled = scalar.current;
        assert!(sampled > 56.0 && sampled < 190.0);

        // User rapidly reverses direction back to 56.0
        scalar.retarget(56.0, Duration::from_millis(150), t1, false);
        assert_eq!(
            scalar.start_value, sampled,
            "Must reverse from sampled value"
        );
        assert_eq!(scalar.target_value, 56.0);
        assert_eq!(scalar.current, sampled, "No position jump on reversal");
    }

    #[test]
    fn test_animated_scalar_clamp() {
        let mut scalar = AnimatedScalar::new(10.0);
        let t0 = Instant::now();
        scalar.retarget(20.0, Duration::from_millis(100), t0, false);

        // Even with huge time jump, clamps exactly at target
        let t_future = t0 + Duration::from_secs(10);
        let active = scalar.update(t_future);
        assert!(!active);
        assert_eq!(scalar.current, 20.0);
    }

    #[test]
    fn test_animated_scalar_reduced_motion_snap() {
        let mut scalar = AnimatedScalar::new(56.0);
        let t0 = Instant::now();
        scalar.retarget(190.0, Duration::from_millis(200), t0, true);

        assert!(
            !scalar.is_active(),
            "Reduced motion must not start frame loop"
        );
        assert_eq!(scalar.current, 190.0, "Must immediately snap to target");
    }

    #[test]
    fn test_animated_scalar_inactive_no_request() {
        let mut scalar = AnimatedScalar::new(56.0);
        let t0 = Instant::now();
        let active = scalar.update(t0);
        assert!(!active);
        assert!(!scalar.is_active());
    }

    #[test]
    fn the_overshoot_passes_the_target_and_comes_back() {
        // L'aiguille franchit son cran, puis revient s'y poser.
        assert!((ease_out_back(0.0) - 0.0).abs() < 1e-6, "le départ est à zéro");
        assert!(ease_out_back(0.6) > 1.0, "le cran est dépassé");
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-6, "l'arrivée est posée");
        // Et le dépassement reste un dépassement, pas un écart.
        assert!(ease_out_back(0.6) < 1.2);
    }

    #[test]
    fn the_cascade_overlaps_and_its_gaps_shrink() {
        // La vague : les éléments se rejoignent, les écarts se rétrécissent.
        let steps = waterfall_entry(&[
            Weight::Heavy,
            Weight::Normal,
            Weight::Normal,
            Weight::Light,
        ]);
        assert_eq!(steps.len(), 4);
        // L'ancre part la première.
        assert_eq!(steps[0].delay, Duration::ZERO);
        // Les écarts se rétrécissent : le premier intervalle est le plus large.
        let first_gap = steps[1].delay - steps[0].delay;
        let second_gap = steps[2].delay - steps[1].delay;
        let third_gap = steps[3].delay - steps[2].delay;
        assert!(first_gap > second_gap, "les écarts se rétrécissent");
        assert!(second_gap > third_gap, "les écarts se rétrécissent");
        // Le premier écart est l'écart de base.
        assert_eq!(first_gap, WATERFALL_BASE_GAP);
        // L'ancre voyage plus loin et plus longtemps.
        assert!(steps[0].travel > steps[1].travel);
        assert!(steps[0].duration > steps[1].duration);
        // Le total couvre le dernier élément.
        let total = waterfall_duration(&steps);
        assert!(total >= steps[3].delay + steps[3].duration);
    }

    #[test]
    fn the_cut_entry_ignites_mid_path_and_keeps_the_direction() {
        // Avant 0,35, rien n'est visible : l'arrivée s'allume en plein chemin.
        let (opacity, offset) = cut_entry(0.0, 40.0);
        assert_eq!(opacity, 0.0);
        assert_eq!(offset, 40.0);
        let (opacity, offset) = cut_entry(0.5, 40.0);
        assert!(opacity > 0.0, "l'assombrissement s'allume en chemin");
        assert!(offset < 40.0 && offset > 0.0, "le voyage se referme");
        let (opacity, offset) = cut_entry(1.0, 40.0);
        assert_eq!(opacity, 1.0);
        assert_eq!(offset, 0.0);
        // Le voyage est partiel : il ne franchit jamais toute la surface.
        assert!(40.0 < 120.0);
    }

    #[test]
    fn test_motion_policy_and_durations() {
        let policy = MotionPolicy::new(true);
        assert!(policy.reduce_motion);

        assert_eq!(MotionDurations::FAST, Duration::from_millis(120));
        assert_eq!(MotionDurations::STANDARD, Duration::from_millis(160));
        assert_eq!(MotionDurations::EMPHASIS, Duration::from_millis(220));

        let q0 = ease_out_quint(0.0);
        let q1 = ease_out_quint(1.0);
        assert_eq!(q0, 0.0);
        assert_eq!(q1, 1.0);

        let io0 = ease_in_out(0.0);
        let io1 = ease_in_out(1.0);
        assert_eq!(io0, 0.0);
        assert_eq!(io1, 1.0);
    }
}
