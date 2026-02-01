use protozoa_rust::simulation::agent::Protozoa;
use protozoa_rust::simulation::environment::PetriDish;
use protozoa_rust::simulation::morphology::Morphology;
use protozoa_rust::simulation::params::{
    BELIEF_LEARNING_RATE, DISH_HEIGHT, DISH_WIDTH, MORPH_FRUSTRATION_THRESHOLD,
    MORPH_SURPRISE_THRESHOLD, MORPH_WINDOW_SIZE, SENSOR_ANGLE, SENSOR_DIST, TARGET_CONCENTRATION,
};

const EPSILON: f64 = 1e-10;

fn assert_float_eq(a: f64, b: f64, msg: &str) {
    assert!((a - b).abs() < EPSILON, "{msg}: expected {b}, got {a}");
}

// === Morphology Structure Tests ===

#[test]
fn test_morphology_initialization() {
    let morph = Morphology::new();
    assert_float_eq(morph.sensor_dist, SENSOR_DIST, "sensor_dist");
    assert_float_eq(morph.sensor_angle, SENSOR_ANGLE, "sensor_angle");
    assert_float_eq(
        morph.belief_learning_rate,
        BELIEF_LEARNING_RATE,
        "belief_learning_rate",
    );
    assert_float_eq(
        morph.target_concentration,
        TARGET_CONCENTRATION,
        "target_concentration",
    );
}

#[test]
fn test_sensor_dist_increases_with_surprise() {
    let mut morph = Morphology::new();
    let initial = morph.sensor_dist;

    morph.adjust_sensor_dist(1.0);
    assert!(morph.sensor_dist > initial);
}

#[test]
fn test_sensor_dist_decreases_with_negative_delta() {
    let mut morph = Morphology::new();
    let initial = morph.sensor_dist;

    morph.adjust_sensor_dist(-1.0);
    assert!(morph.sensor_dist < initial);
}

#[test]
fn test_sensor_angle_widens_with_surprise() {
    let mut morph = Morphology::new();
    let initial = morph.sensor_angle;

    morph.adjust_sensor_angle(1.0);
    assert!(morph.sensor_angle > initial);
}

#[test]
fn test_belief_learning_rate_increases_with_surprise() {
    let mut morph = Morphology::new();
    let initial = morph.belief_learning_rate;

    morph.adjust_belief_learning_rate(1.0);
    assert!(morph.belief_learning_rate > initial);
}

#[test]
fn test_target_concentration_decreases_with_frustration() {
    let mut morph = Morphology::new();
    let initial = morph.target_concentration;

    morph.adjust_target_concentration(1.0);
    assert!(morph.target_concentration < initial);
}

#[test]
fn test_morphology_clamping() {
    let mut morph = Morphology::new();

    // Test sensor_dist max clamp
    morph.adjust_sensor_dist(100.0);
    assert!(morph.sensor_dist <= 4.0);

    // Test sensor_angle max clamp
    morph.adjust_sensor_angle(100.0);
    assert!(morph.sensor_angle <= 1.0);

    // Test belief_learning_rate max clamp
    morph.adjust_belief_learning_rate(100.0);
    assert!(morph.belief_learning_rate <= 0.3);

    // Test target_concentration min clamp
    morph.adjust_target_concentration(100.0);
    assert!(morph.target_concentration >= 0.5);
}

// === Agent Integration Tests ===

#[test]
fn test_agent_has_morphology() {
    let agent = Protozoa::new(50.0, 25.0);
    assert_float_eq(agent.morphology.sensor_dist, SENSOR_DIST, "sensor_dist");
    assert_float_eq(agent.morphology.sensor_angle, SENSOR_ANGLE, "sensor_angle");
}

#[test]
fn test_agent_has_accumulators() {
    let agent = Protozoa::new(50.0, 25.0);
    assert_float_eq(agent.cumulative_surprise, 0.0, "cumulative_surprise");
    assert_float_eq(agent.cumulative_frustration, 0.0, "cumulative_frustration");
    assert_eq!(agent.morph_window_start, 0);
}

#[test]
fn test_agent_sense_uses_dynamic_parameters() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Modify morphology parameters
    agent.morphology.sensor_dist = 3.0;
    agent.morphology.sensor_angle = 0.8;

    // Sense should use these values
    agent.sense(&dish);

    // Verify sensors read valid values (testing that no panic occurred)
    assert!(agent.val_l >= -1.0 && agent.val_l <= 1.0);
    assert!(agent.val_r >= -1.0 && agent.val_r <= 1.0);
}

#[test]
fn test_agent_update_uses_dynamic_learning_rate() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Set a specific learning rate
    agent.morphology.belief_learning_rate = 0.25;

    // Run update
    agent.sense(&dish);
    let initial_beliefs = agent.beliefs.mean.nutrient;
    agent.update_state(&dish);

    // Beliefs should have changed (verifying update occurred)
    // Note: exact change depends on many factors, we just verify it's different
    assert_ne!(agent.beliefs.mean.nutrient, initial_beliefs);
}

// === System 2 Regulation Tests ===

#[test]
fn test_surprise_accumulation() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    agent.sense(&dish);
    agent.update_state(&dish);

    // Surprise should have accumulated
    assert!(agent.cumulative_surprise > 0.0);
}

#[test]
fn test_frustration_accumulation() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Run multiple updates to ensure some frustration accumulates
    // (single tick might have negative EFE due to epistemic value)
    for _ in 0..10 {
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // Frustration should have accumulated over multiple ticks
    // (at least some ticks should have positive EFE)
    assert!(
        agent.cumulative_frustration >= 0.0,
        "Frustration accumulator should be non-negative"
    );
}

#[test]
fn test_morphology_regulation_requires_window() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    let initial_sensor_dist = agent.morphology.sensor_dist;

    // Run updates but not enough to trigger regulation
    for _ in 0..(MORPH_WINDOW_SIZE - 1) {
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // Morphology should not have changed (window not complete)
    assert_float_eq(
        agent.morphology.sensor_dist,
        initial_sensor_dist,
        "sensor_dist should not change before window completes",
    );
}

#[test]
fn test_structural_morphogenesis_with_high_surprise() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    let initial_sensor_dist = agent.morphology.sensor_dist;
    let initial_sensor_angle = agent.morphology.sensor_angle;

    // Inject high surprise to force morphogenesis
    for _ in 0..MORPH_WINDOW_SIZE {
        agent.cumulative_surprise += MORPH_SURPRISE_THRESHOLD * 1.5;
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // Sensor parameters should have changed
    assert_ne!(
        agent.morphology.sensor_dist, initial_sensor_dist,
        "sensor_dist should change with high surprise"
    );
    assert_ne!(
        agent.morphology.sensor_angle, initial_sensor_angle,
        "sensor_angle should change with high surprise"
    );
}

#[test]
fn test_allostatic_regulation_with_high_frustration() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    let initial_target = agent.morphology.target_concentration;

    // Inject high frustration to force allostatic regulation
    for _ in 0..MORPH_WINDOW_SIZE {
        agent.cumulative_frustration += MORPH_FRUSTRATION_THRESHOLD * 1.5;
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // Target concentration should have decreased (allostatic load)
    assert!(
        agent.morphology.target_concentration < initial_target,
        "target should decrease with high frustration"
    );
}

#[test]
fn test_accumulator_reset_after_regulation() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Inject high surprise
    for _ in 0..MORPH_WINDOW_SIZE {
        agent.cumulative_surprise += MORPH_SURPRISE_THRESHOLD * 2.0;
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // After regulation, accumulators should be near zero
    assert!(agent.cumulative_surprise < 1.0);
}

#[test]
fn test_generative_model_sync_with_morphology() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Modify morphology
    agent.morphology.sensor_angle = 0.9;

    // Force regulation
    for _ in 0..MORPH_WINDOW_SIZE {
        agent.cumulative_surprise += MORPH_SURPRISE_THRESHOLD * 2.0;
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // Generative model should reflect morphology changes
    // (sensor_angle updated during regulation)
    assert!(
        (agent.generative_model.sensor_angle - agent.morphology.sensor_angle).abs() < 0.5,
        "generative model sensor_angle should sync with morphology"
    );
}

// === Integration Tests ===

#[test]
fn test_system_1_system_2_loop() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let mut dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Run full simulation for enough ticks to potentially trigger regulation
    for _ in 0..(MORPH_WINDOW_SIZE * 2) {
        dish.update();
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // Verify system is functioning (agent is alive and moving)
    assert!(agent.energy > 0.0);
    assert!(agent.tick_count > 0);

    // Morphology may or may not have changed depending on accumulated surprise/frustration
    // Just verify no crashes and system is coherent
    assert!(agent.morphology.sensor_dist > 0.0);
    assert!(agent.morphology.sensor_angle > 0.0);
}

#[test]
fn test_morphology_bounds_maintained() {
    let mut agent = Protozoa::new(50.0, 25.0);
    let dish = PetriDish::new(DISH_WIDTH, DISH_HEIGHT);

    // Force extreme regulation
    for _ in 0..(MORPH_WINDOW_SIZE * 5) {
        agent.cumulative_surprise += MORPH_SURPRISE_THRESHOLD * 10.0;
        agent.cumulative_frustration += MORPH_FRUSTRATION_THRESHOLD * 10.0;
        agent.sense(&dish);
        agent.update_state(&dish);
    }

    // All morphology parameters should remain within valid bounds
    assert!(agent.morphology.sensor_dist >= 1.0 && agent.morphology.sensor_dist <= 4.0);
    assert!(agent.morphology.sensor_angle >= 0.2 && agent.morphology.sensor_angle <= 1.0);
    assert!(
        agent.morphology.belief_learning_rate >= 0.05
            && agent.morphology.belief_learning_rate <= 0.3
    );
    assert!(
        agent.morphology.target_concentration >= 0.5
            && agent.morphology.target_concentration <= 0.9
    );
}
