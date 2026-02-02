# Protozoa
**Continuous Active Inference Simulation**

Protozoa is a zero-player biological simulation where a single-cell agent navigates a nutrient-rich petri dish using the **Free Energy Principle (FEP)**. Unlike traditional AI that follows hard-coded rules, this agent operates by minimizing the difference between its genetic expectation (Homeostasis) and its actual sensory input.

![Platform](https://img.shields.io/badge/Platform-Linux%20Terminal-black)
![Language](https://img.shields.io/badge/Language-Rust-orange)
![License](https://img.shields.io/badge/License-AGPLv3-green)

## ✨ Features
*   **Genuine Active Inference:** Gaussian beliefs q(s) = N(μ, Σ), Variational Free Energy minimization, Expected Free Energy for action selection.
*   **Stereo Vision:** Two chemical sensors detect continuous gradients.
*   **Multi-Layer Memory:**
    *   **Short-term:** Ring buffer of 32 recent experiences
    *   **Long-term:** 20×10 spatial grid learning nutrient expectations (Welford's algorithm)
    *   **Episodic:** Up to 8 remembered landmarks with reliability decay
*   **MCTS Planning:** Monte Carlo Tree Search with Expected Free Energy (pragmatic + epistemic value).
*   **Goal-Directed Navigation:** Returns to remembered food sources when energy is low.
*   **Morphogenetic Computation:** Endogenous structural evolution via System 2 regulator, satisfying axioms A1-A6 for true morphological computation.
*   **High Performance:** Parallelized field rendering using `rayon`.
*   **Static Binary:** Ship a single executable with no external dependencies.
*   **Dynamic Environment:** Food sources decay, move (Brownian motion), and regrow.
*   **Metabolic System:** Managing energy (ATP) is crucial; exhaustion leads to death spirals.
*   **Emergent Behavior:** Watch the agent panic, tumble, sprint, and graze without explicit instructions.

## 🚀 Getting Started

### Prerequisites
*   **Rust Toolchain:** Install via [rustup.rs](https://rustup.rs/).

### Running the Simulation
Clone the repository and run using `cargo`:

```bash
git clone https://github.com/ahenkes1/protozoa.git
cd protozoa/protozoa_rust
cargo run --release
```

*(Note: Use `--release` for optimal frame rates)*

### Static Compilation (Linux)
To build a dependency-free static binary (MUSL):

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```
The binary will be located at `target/x86_64-unknown-linux-musl/release/protozoa_rust`.

## 🎮 Controls
This is a **zero-player game**, meaning you watch life unfold.
*   **`q`**: Quit the simulation.

## 🛠️ Development

### Project Structure
*   `src/main.rs`: Entry point and visualization loop (`ratatui` + `crossterm`).
*   `src/simulation/`: Core logic module.
    *   `agent.rs`: Continuous Active Inference with Gaussian beliefs and VFE/EFE.
    *   `environment.rs`: Petri Dish and Nutrient physics.
    *   `params.rs`: All configurable hyperparameters.
    *   `inference/`: Active Inference engine (beliefs, generative model, free energy, precision).
    *   `memory/`: Memory systems (ring buffer, spatial grid, episodic landmarks).
    *   `planning/`: MCTS planner with Expected Free Energy evaluation.
*   `src/ui/`: Rendering module.
    *   `field.rs`: Parallelized grid computation (`rayon`).
    *   `render.rs`: TUI rendering with sidebar dashboard layout.

### Dashboard Layout
The TUI displays a cognitive dashboard with sidebar layout:

```
┌──────────────────────────────┬─────────────┐
│                              │  Agent      │
│                              │  (metrics)  │
│        Petri Dish            ├─────────────┤
│        (~70% width)          │  MCTS       │
│                              │  (planning) │
│        ASCII visualization   ├─────────────┤
│        of environment        │  Landmarks  │
│                              │  (episodic) │
│                              ├─────────────┤
│                              │  Spatial    │
│                              │  (priors)   │
└──────────────────────────────┴─────────────┘
```

*   **Petri Dish (left):** ASCII visualization of nutrient concentrations and agent position
*   **Agent panel:** Energy bar, mode, prediction error, precision, sensors, temporal gradient
*   **MCTS panel:** Best action, Expected Free Energy breakdown (pragmatic/epistemic)
*   **Landmarks panel:** Remembered food locations with reliability and visit counts
*   **Spatial Memory:** Heatmap of learned nutrient expectations (auto-compresses for narrow terminals)

### Configuration
All simulation parameters are defined in `src/simulation/params.rs`:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `TARGET_CONCENTRATION` | 0.8 | Homeostatic set-point |
| `LEARNING_RATE` | 0.15 | Gradient descent step size |
| `MAX_SPEED` | 1.5 | Maximum movement speed |
| `PANIC_THRESHOLD` | -0.01 | Temporal gradient trigger |
| `EXHAUSTION_THRESHOLD` | 0.01 | Energy level for exhaustion |
| `EXPLORATION_SCALE` | 0.3 | Bonus for exploring uncertain regions |
| `MAX_LANDMARKS` | 8 | Max remembered food locations |
| `LANDMARK_THRESHOLD` | 0.7 | Min nutrient to store landmark |
| `MCTS_ROLLOUTS` | 50 | Trajectories per planning step |
| `MCTS_DEPTH` | 10 | Lookahead depth for planning |
| `PLANNING_WEIGHT` | 0.3 | Blend of planned vs reactive control |
| `BELIEF_LEARNING_RATE` | 0.15 | VFE gradient descent step size |
| `INITIAL_SENSORY_PRECISION` | 5.0 | Starting sensor precision |
| `NUTRIENT_PRIOR_PRECISION` | 2.0 | Strength of nutrient preference |
| `SURPRISE_THRESHOLD` | 10.0 | VFE integral trigger for morphogenesis |
| `FRUSTRATION_THRESHOLD` | 5.0 | EFE integral trigger for morphogenesis |
| `SENSOR_DIST_ENERGY_COST` | 0.01 | Energy cost per unit sensor distance change |
| `SENSOR_ANGLE_ENERGY_COST` | 0.005 | Energy cost per unit sensor angle change |
| `LEARNING_RATE_ENERGY_COST` | 0.002 | Energy cost per unit learning rate change |
| `MAX_COMPLEXITY` | 10.0 | Soft limit on structural complexity |
| `COMPLEXITY_ENERGY_COST_MULTIPLIER` | 2.0 | Energy penalty multiplier for high complexity |

### Running Tests
```bash
cargo test  # Runs 136 tests across 9 test files
```

### Code Quality
We enforce strict linting and formatting (also in CI):
```bash
cargo fmt --check
cargo clippy -- -D warnings
```

## 🧠 How it Works

### Continuous Active Inference
The agent maintains Gaussian beliefs q(s) = N(μ, Σ) over hidden states (nutrient, position, heading) and updates them by minimizing Variational Free Energy:

$$
F = \frac{1}{2}(o - g(\mu))^T \Pi_o (o - g(\mu)) + \frac{1}{2}(\mu - \eta)^T \Pi_\eta (\mu - \eta)
$$

### Each Tick
1.  **Sense:** Update sensory inputs from environment
2.  **Infer:** Gradient descent on VFE updates beliefs: dμ/dt = -∂F/∂μ
3.  **Learn:** Update sensory precision from prediction errors
4.  **Plan:** Evaluate actions by Expected Free Energy, select minimum
5.  **Act:** Blend reactive control + planned action + exploration + goal attraction
6.  **Metabolize:** Update energy and accumulate stress for morphogenesis
7.  **Morphogen:** System 2 regulator triggers endogenous structural changes
8.  **Panic:** Random tumble if conditions worsen rapidly (temporal gradient)

### Action Selection via Expected Free Energy
```
G(π) = Risk + Ambiguity - Epistemic
Risk = deviation from preferred nutrient (0.8)
Ambiguity = uncertainty in predictions
Epistemic = information gain (uncertainty reduction)
```
Lower EFE is better (we minimize G).

### Memory Systems
- **Short-term:** 32-element ring buffer of recent experiences
- **Long-term:** 20×10 grid learns nutrient expectations via Welford's algorithm
- **Episodic:** Stores up to 8 high-nutrient landmarks with reliability decay

### Morphogenetic Computation
The agent implements morphological/morphogenetic/whatever were going with..... computation where structure emerges endogenously from computation, satisfying axioms A1-A6:

- **System Model M = (Σ, Φ, R, C, B):**
  - Σ: Substrate (morphology, generative model, memory)
  - Φ: State fields (beliefs, energy, accumulators)
  - R: Rules (VFE/EFE minimization, regulator triggers)
  - C: Invariants (energy conservation, physiological limits)
  - B: Boundary drives (homeostasis, stress thresholds)

- **Axioms A1-A6:**
  - A1: No external schedulers - changes triggered by internal accumulators
  - A2: Endogenous substrate emergence - structure evolves from dynamics
  - A3: Causal feedback - morphology influences future computation
  - A4: Open configuration space - unbounded potential complexity
  - A5: Causal reality - changes incur metabolic costs
  - A6: Continuous internal generation - ongoing structural evolution

- **System 2 Regulator:** Monitors ∫VFE dt (surprise) and ∫EFE dt (frustration) to trigger morphological changes with energy costs and complexity constraints.
