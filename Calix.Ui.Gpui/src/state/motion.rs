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
