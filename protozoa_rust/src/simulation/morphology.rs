//! Morphology: Dynamic agent parameters that adapt via System 2 regulation.
//!
//! The Morphology struct represents the agent's malleable physical parameters
//! that can change in response to accumulated surprise and frustration.
//!
//! # System 2 Regulation
//! - **Structural Morphogenesis**: Adjust sensor geometry based on surprise (VFE)
//! - **Allostatic Regulation**: Adjust homeostatic targets based on frustration (EFE)

use crate::simulation::params::{
    BELIEF_LEARNING_RATE, SENSOR_ANGLE, SENSOR_DIST, TARGET_CONCENTRATION,
};

/// Morphological parameters that can adapt over time.
///
/// These parameters define the agent's physical structure (sensor geometry)
/// and cognitive parameters (learning rates, homeostatic targets).
#[derive(Clone, Debug)]
pub struct Morphology {
    /// Distance from body center to chemical sensors
    pub sensor_dist: f64,
    /// Angle offset of sensors from heading (stereo spread)
    pub sensor_angle: f64,
    /// Learning rate for belief updates via VFE gradient descent
    pub belief_learning_rate: f64,
    /// Target nutrient concentration (homeostatic set-point)
    pub target_concentration: f64,
}

impl Default for Morphology {
    fn default() -> Self {
        Self::new()
    }
}

impl Morphology {
    /// Create a new morphology with default parameters from PARAMS.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sensor_dist: SENSOR_DIST,
            sensor_angle: SENSOR_ANGLE,
            belief_learning_rate: BELIEF_LEARNING_RATE,
            target_concentration: TARGET_CONCENTRATION,
        }
    }

    /// Adjust sensor distance based on accumulated surprise.
    ///
    /// High surprise → Increase sensor distance to sample larger gradients
    /// Low surprise → Decrease sensor distance for finer local sensing
    pub fn adjust_sensor_dist(&mut self, surprise_delta: f64) {
        const MIN_SENSOR_DIST: f64 = 1.0;
        const MAX_SENSOR_DIST: f64 = 4.0;
        const SENSOR_DIST_RATE: f64 = 0.1;

        self.sensor_dist += SENSOR_DIST_RATE * surprise_delta;
        self.sensor_dist = self.sensor_dist.clamp(MIN_SENSOR_DIST, MAX_SENSOR_DIST);
    }

    /// Adjust sensor angle based on accumulated surprise.
    ///
    /// High surprise → Widen stereo angle for better gradient detection
    /// Low surprise → Narrow angle for focused sensing
    pub fn adjust_sensor_angle(&mut self, surprise_delta: f64) {
        const MIN_SENSOR_ANGLE: f64 = 0.2; // ~11.5 degrees
        const MAX_SENSOR_ANGLE: f64 = 1.0; // ~57 degrees
        const SENSOR_ANGLE_RATE: f64 = 0.05;

        self.sensor_angle += SENSOR_ANGLE_RATE * surprise_delta;
        self.sensor_angle = self.sensor_angle.clamp(MIN_SENSOR_ANGLE, MAX_SENSOR_ANGLE);
    }

    /// Adjust belief learning rate based on accumulated surprise.
    ///
    /// High surprise → Increase learning rate to adapt faster
    /// Low surprise → Decrease learning rate for stability
    pub fn adjust_belief_learning_rate(&mut self, surprise_delta: f64) {
        const MIN_LEARNING_RATE: f64 = 0.05;
        const MAX_LEARNING_RATE: f64 = 0.3;
        const LEARNING_RATE_RATE: f64 = 0.01;

        self.belief_learning_rate += LEARNING_RATE_RATE * surprise_delta;
        self.belief_learning_rate = self
            .belief_learning_rate
            .clamp(MIN_LEARNING_RATE, MAX_LEARNING_RATE);
    }

    /// Adjust target concentration based on accumulated frustration.
    ///
    /// High frustration → Lower target (allostatic load)
    /// Low frustration → Restore target toward ideal
    pub fn adjust_target_concentration(&mut self, frustration_delta: f64) {
        const MIN_TARGET: f64 = 0.5;
        const MAX_TARGET: f64 = 0.9;
        const TARGET_RATE: f64 = 0.02;
        const IDEAL_TARGET: f64 = TARGET_CONCENTRATION;

        // Frustration lowers target (allostatic load)
        // Recovery slowly restores toward ideal
        if frustration_delta > 0.0 {
            self.target_concentration -= TARGET_RATE * frustration_delta;
        } else {
            // Slowly recover toward ideal when not frustrated
            let recovery = (IDEAL_TARGET - self.target_concentration) * 0.05;
            self.target_concentration += recovery;
        }

        self.target_concentration = self.target_concentration.clamp(MIN_TARGET, MAX_TARGET);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphology_default() {
        let morph = Morphology::new();
        assert_eq!(morph.sensor_dist, SENSOR_DIST);
        assert_eq!(morph.sensor_angle, SENSOR_ANGLE);
        assert_eq!(morph.belief_learning_rate, BELIEF_LEARNING_RATE);
        assert_eq!(morph.target_concentration, TARGET_CONCENTRATION);
    }

    #[test]
    fn test_adjust_sensor_dist() {
        let mut morph = Morphology::new();
        let initial = morph.sensor_dist;

        // High surprise increases distance
        morph.adjust_sensor_dist(1.0);
        assert!(morph.sensor_dist > initial);

        // Should clamp at max
        morph.adjust_sensor_dist(100.0);
        assert!(morph.sensor_dist <= 4.0);
    }

    #[test]
    fn test_adjust_sensor_angle() {
        let mut morph = Morphology::new();
        let initial = morph.sensor_angle;

        // High surprise widens angle
        morph.adjust_sensor_angle(1.0);
        assert!(morph.sensor_angle > initial);

        // Should clamp at max
        morph.adjust_sensor_angle(100.0);
        assert!(morph.sensor_angle <= 1.0);
    }

    #[test]
    fn test_adjust_belief_learning_rate() {
        let mut morph = Morphology::new();
        let initial = morph.belief_learning_rate;

        // High surprise increases learning rate
        morph.adjust_belief_learning_rate(1.0);
        assert!(morph.belief_learning_rate > initial);

        // Should clamp at max
        morph.adjust_belief_learning_rate(100.0);
        assert!(morph.belief_learning_rate <= 0.3);
    }

    #[test]
    fn test_adjust_target_concentration() {
        let mut morph = Morphology::new();
        let initial = morph.target_concentration;

        // High frustration lowers target
        morph.adjust_target_concentration(1.0);
        assert!(morph.target_concentration < initial);

        // Should clamp at min
        morph.adjust_target_concentration(100.0);
        assert!(morph.target_concentration >= 0.5);

        // Recovery restores toward ideal
        for _ in 0..100 {
            morph.adjust_target_concentration(-0.1);
        }
        assert!(morph.target_concentration > 0.5);
    }
}
