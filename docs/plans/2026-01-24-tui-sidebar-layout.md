# TUI Sidebar Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restructure the TUI dashboard from 2x2 quadrants to main+sidebar layout, maximizing Petri Dish visualization area.

**Architecture:** Replace `compute_quadrant_layout` with `compute_sidebar_layout` that splits 70%/30% horizontally. Petri Dish takes the left 70% full-height. Sidebar stacks 4 panels vertically: Metrics (extracted from overlay), MCTS, Landmarks, Spatial Memory (with dynamic compression).

**Tech Stack:** Rust, ratatui (TUI framework), existing `DashboardState` unchanged.

---

## Task 1: Add `compute_sidebar_layout` Function

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs:14-35`
- Test: `protozoa_rust/src/ui/render.rs` (inline test module)

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `render.rs`:

```rust
#[test]
fn test_compute_sidebar_layout() {
    use ratatui::layout::Rect;

    let area = Rect::new(0, 0, 100, 40);
    let (main, sidebar) = compute_sidebar_layout(area);

    // Main panel should be ~70% width
    assert!(main.width >= 68 && main.width <= 72, "main width: {}", main.width);
    assert_eq!(main.height, 40);
    assert_eq!(main.x, 0);

    // Sidebar should be ~30% width
    assert!(sidebar.len() == 4, "should have 4 sidebar panels");
    assert!(sidebar[0].width >= 28 && sidebar[0].width <= 32, "sidebar width: {}", sidebar[0].width);

    // Sidebar panels should stack vertically
    assert_eq!(sidebar[0].y, 0); // Metrics at top
    assert!(sidebar[1].y > sidebar[0].y); // MCTS below Metrics
    assert!(sidebar[2].y > sidebar[1].y); // Landmarks below MCTS
    assert!(sidebar[3].y > sidebar[2].y); // Spatial below Landmarks
}
```

**Step 2: Run test to verify it fails**

Run: `cd protozoa_rust && cargo test test_compute_sidebar_layout -- --nocapture`

Expected: FAIL with "cannot find function `compute_sidebar_layout`"

**Step 3: Write minimal implementation**

Add new function after imports (before `compute_quadrant_layout`):

```rust
/// Computes the main + sidebar layout for the dashboard.
/// Returns (main_area, sidebar_panels) where sidebar_panels is [Metrics, MCTS, Landmarks, Spatial].
#[must_use]
pub fn compute_sidebar_layout(area: Rect) -> (Rect, Vec<Rect>) {
    // Horizontal split: 70% main, 30% sidebar
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let main = horizontal[0];

    // Sidebar vertical split: fixed heights for top 3, remaining for Spatial
    let sidebar_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Metrics
            Constraint::Length(9),  // MCTS
            Constraint::Length(12), // Landmarks
            Constraint::Min(0),     // Spatial (remaining)
        ])
        .split(horizontal[1]);

    (main, sidebar_panels.to_vec())
}
```

**Step 4: Run test to verify it passes**

Run: `cd protozoa_rust && cargo test test_compute_sidebar_layout -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
cd protozoa_rust && git add src/ui/render.rs && git commit -m "feat(ui): add compute_sidebar_layout function"
```

---

## Task 2: Add `draw_metrics_panel` Function

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs`

**Step 1: Write the failing test**

Add to tests module:

```rust
#[test]
fn test_draw_metrics_panel_renders_without_panic() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::ui::DashboardState;
    use crate::simulation::agent::AgentMode;
    use crate::simulation::memory::CellPrior;

    let backend = TestBackend::new(30, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = DashboardState {
        x: 50.0, y: 25.0, angle: 1.0, speed: 0.5,
        energy: 0.8, mode: AgentMode::Exploring,
        prediction_error: -0.2, precision: 5.0,
        sensor_left: 0.6, sensor_right: 0.5,
        temporal_gradient: 0.03,
        spatial_grid: vec![CellPrior::default(); 200],
        grid_width: 20, grid_height: 10,
        plan_details: vec![],
        ticks_until_replan: 15,
        landmarks: vec![],
        landmark_count: 0,
        nav_target_index: None,
    };

    terminal.draw(|f| {
        let area = Rect::new(0, 0, 25, 8);
        draw_metrics_panel(f, area, &state);
    }).unwrap();

    // If we get here without panic, the test passes
}
```

**Step 2: Run test to verify it fails**

Run: `cd protozoa_rust && cargo test test_draw_metrics_panel -- --nocapture`

Expected: FAIL with "cannot find function `draw_metrics_panel`"

**Step 3: Write minimal implementation**

Add new function after `draw_petri_dish_panel`:

```rust
fn draw_metrics_panel(f: &mut Frame, area: Rect, state: &DashboardState) {
    let block = Block::default().title(" Agent ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let angle_deg = state.angle.to_degrees();
    let lines = format_metrics_overlay(
        state.energy,
        state.mode,
        state.prediction_error,
        state.precision,
        state.speed,
        angle_deg,
        state.sensor_left,
        state.sensor_right,
        state.temporal_gradient,
    );

    let text: Vec<Line> = lines
        .into_iter()
        .map(|s| Line::from(Span::styled(s, Style::default().add_modifier(Modifier::BOLD))))
        .collect();
    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
}
```

**Step 4: Run test to verify it passes**

Run: `cd protozoa_rust && cargo test test_draw_metrics_panel -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
cd protozoa_rust && git add src/ui/render.rs && git commit -m "feat(ui): add draw_metrics_panel function"
```

---

## Task 3: Add `compress_spatial_grid` Function

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs`

**Step 1: Write the failing test**

Add to tests module:

```rust
#[test]
fn test_compress_spatial_grid_no_compression_needed() {
    use crate::simulation::memory::CellPrior;

    // 4x2 grid, target width 4 (no compression)
    let cells: Vec<CellPrior> = (0..8)
        .map(|i| {
            let mut c = CellPrior::default();
            c.mean = i as f64 * 0.1;
            c
        })
        .collect();

    let result = compress_spatial_grid(&cells, 4, 2, 4);
    assert_eq!(result.len(), 8);
    assert!((result[0].mean - 0.0).abs() < 0.001);
    assert!((result[3].mean - 0.3).abs() < 0.001);
}

#[test]
fn test_compress_spatial_grid_halves_width() {
    use crate::simulation::memory::CellPrior;

    // 4x2 grid, compress to width 2
    // Row 0: [0.0, 0.2, 0.4, 0.6] -> [0.1, 0.5]
    // Row 1: [0.4, 0.6, 0.8, 1.0] -> [0.5, 0.9]
    let mut cells = Vec::new();
    for row in 0..2 {
        for col in 0..4 {
            let mut c = CellPrior::default();
            c.mean = (row * 4 + col) as f64 * 0.2;
            cells.push(c);
        }
    }

    let result = compress_spatial_grid(&cells, 4, 2, 2);
    assert_eq!(result.len(), 4); // 2x2 grid

    // Check averaged values
    assert!((result[0].mean - 0.1).abs() < 0.001, "got {}", result[0].mean); // avg(0.0, 0.2)
    assert!((result[1].mean - 0.5).abs() < 0.001, "got {}", result[1].mean); // avg(0.4, 0.6)
}
```

**Step 2: Run test to verify it fails**

Run: `cd protozoa_rust && cargo test test_compress_spatial_grid -- --nocapture`

Expected: FAIL with "cannot find function `compress_spatial_grid`"

**Step 3: Write minimal implementation**

Add function before `render_spatial_grid_lines`:

```rust
/// Compresses spatial grid horizontally by averaging adjacent cells.
/// If target_width >= orig_width, returns a copy unchanged.
#[must_use]
#[allow(clippy::cast_precision_loss)]
fn compress_spatial_grid(
    cells: &[CellPrior],
    orig_width: usize,
    orig_height: usize,
    target_width: usize,
) -> Vec<CellPrior> {
    if target_width >= orig_width {
        return cells.to_vec();
    }

    let mut result = Vec::with_capacity(target_width * orig_height);
    let ratio = orig_width as f64 / target_width as f64;

    for row in 0..orig_height {
        for target_col in 0..target_width {
            let start_col = (target_col as f64 * ratio).floor() as usize;
            let end_col = (((target_col + 1) as f64) * ratio).floor() as usize;
            let end_col = end_col.min(orig_width);

            let mut sum_mean = 0.0;
            let mut count = 0;

            for col in start_col..end_col {
                let idx = row * orig_width + col;
                if let Some(cell) = cells.get(idx) {
                    sum_mean += cell.mean;
                    count += 1;
                }
            }

            let mut compressed = CellPrior::default();
            if count > 0 {
                compressed.mean = sum_mean / count as f64;
            }
            result.push(compressed);
        }
    }

    result
}
```

**Step 4: Run test to verify it passes**

Run: `cd protozoa_rust && cargo test test_compress_spatial_grid -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
cd protozoa_rust && git add src/ui/render.rs && git commit -m "feat(ui): add compress_spatial_grid function"
```

---

## Task 4: Update `draw_spatial_grid_panel` to Use Compression

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs:110-140`

**Step 1: Write the failing test**

Add to tests module:

```rust
#[test]
fn test_spatial_grid_panel_handles_narrow_width() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::ui::DashboardState;
    use crate::simulation::agent::AgentMode;
    use crate::simulation::memory::CellPrior;

    let backend = TestBackend::new(15, 15); // Narrow terminal
    let mut terminal = Terminal::new(backend).unwrap();

    let state = DashboardState {
        x: 50.0, y: 25.0, angle: 1.0, speed: 0.5,
        energy: 0.8, mode: AgentMode::Exploring,
        prediction_error: -0.2, precision: 5.0,
        sensor_left: 0.6, sensor_right: 0.5,
        temporal_gradient: 0.03,
        spatial_grid: vec![CellPrior::default(); 200], // 20x10 grid
        grid_width: 20, grid_height: 10,
        plan_details: vec![],
        ticks_until_replan: 15,
        landmarks: vec![],
        landmark_count: 0,
        nav_target_index: None,
    };

    // Should not panic even with narrow width
    terminal.draw(|f| {
        let area = Rect::new(0, 0, 15, 12);
        draw_spatial_grid_panel(f, area, &state);
    }).unwrap();
}
```

**Step 2: Run test to verify it passes (existing function should work)**

Run: `cd protozoa_rust && cargo test test_spatial_grid_panel_handles_narrow_width -- --nocapture`

Expected: PASS (but grid may overflow visually)

**Step 3: Update implementation to use compression**

Modify `draw_spatial_grid_panel` function:

```rust
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn draw_spatial_grid_panel(f: &mut Frame, area: Rect, state: &DashboardState) {
    let block = Block::default()
        .title(" Spatial Memory ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Determine target width based on available space
    let target_width = (inner.width as usize).min(state.grid_width);

    // Compress grid if needed
    let display_cells = if target_width < state.grid_width {
        compress_spatial_grid(&state.spatial_grid, state.grid_width, state.grid_height, target_width)
    } else {
        state.spatial_grid.clone()
    };

    let display_width = target_width.min(state.grid_width);

    // Calculate agent's grid cell (in compressed coordinates)
    let compression_ratio = state.grid_width as f64 / display_width as f64;
    let agent_col = ((state.x / 100.0) * state.grid_width as f64 / compression_ratio).floor() as usize;
    let agent_row = ((state.y / 50.0) * state.grid_height as f64).floor() as usize;
    let agent_cell = Some((
        agent_row.min(state.grid_height.saturating_sub(1)),
        agent_col.min(display_width.saturating_sub(1)),
    ));

    let lines = render_spatial_grid_lines(
        &display_cells,
        display_width,
        state.grid_height,
        agent_cell,
    );
    let text: Vec<Line> = lines
        .into_iter()
        .map(|s| Line::from(Span::raw(s)))
        .collect();
    let grid = Paragraph::new(text);
    f.render_widget(grid, inner);
}
```

**Step 4: Run test to verify it still passes**

Run: `cd protozoa_rust && cargo test test_spatial_grid_panel -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
cd protozoa_rust && git add src/ui/render.rs && git commit -m "feat(ui): add grid compression to draw_spatial_grid_panel"
```

---

## Task 5: Simplify `draw_petri_dish_panel` (Remove Overlay)

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs:54-108`

**Step 1: Note existing behavior**

The current `draw_petri_dish_panel` renders both the field and an overlay. We need to remove the overlay rendering.

**Step 2: Update implementation**

Replace the entire `draw_petri_dish_panel` function:

```rust
fn draw_petri_dish_panel(f: &mut Frame, area: Rect, grid_lines: Vec<String>) {
    let block = Block::default().title(" Petri Dish ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render field only (no overlay - metrics moved to sidebar)
    let text: Vec<Line> = grid_lines
        .into_iter()
        .map(|s| Line::from(Span::raw(s)))
        .collect();
    let field = Paragraph::new(text);
    f.render_widget(field, inner);
}
```

**Step 3: Run tests to check for breakage**

Run: `cd protozoa_rust && cargo test -- --nocapture`

Expected: May fail if tests depend on old signature. Fix in next step.

**Step 4: Commit**

```bash
cd protozoa_rust && git add src/ui/render.rs && git commit -m "refactor(ui): remove overlay from draw_petri_dish_panel"
```

---

## Task 6: Update `draw_dashboard` to Use Sidebar Layout

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs:37-52`

**Step 1: Write the failing test**

Add to tests module:

```rust
#[test]
fn test_draw_dashboard_uses_sidebar_layout() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::ui::DashboardState;
    use crate::simulation::agent::AgentMode;
    use crate::simulation::memory::CellPrior;

    let backend = TestBackend::new(100, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = DashboardState {
        x: 50.0, y: 25.0, angle: 1.0, speed: 0.5,
        energy: 0.8, mode: AgentMode::Exploring,
        prediction_error: -0.2, precision: 5.0,
        sensor_left: 0.6, sensor_right: 0.5,
        temporal_gradient: 0.03,
        spatial_grid: vec![CellPrior::default(); 200],
        grid_width: 20, grid_height: 10,
        plan_details: vec![],
        ticks_until_replan: 15,
        landmarks: vec![],
        landmark_count: 0,
        nav_target_index: None,
    };

    let grid_lines: Vec<String> = (0..30).map(|_| ".".repeat(60)).collect();

    terminal.draw(|f| {
        draw_dashboard(f, grid_lines.clone(), &state);
    }).unwrap();

    // Verify buffer has content in expected regions
    let buffer = terminal.backend().buffer();

    // Check "Petri Dish" title is in top-left area
    let petri_title_found = (0..20).any(|x| {
        buffer.cell((x, 0)).map(|c| c.symbol()).unwrap_or("") == "P"
    });
    assert!(petri_title_found, "Petri Dish title should be on left side");

    // Check "Agent" title is in right sidebar area (x > 60)
    let agent_title_found = (60..100).any(|x| {
        buffer.cell((x, 0)).map(|c| c.symbol()).unwrap_or("") == "A"
    });
    assert!(agent_title_found, "Agent panel title should be on right side");
}
```

**Step 2: Run test to verify it fails**

Run: `cd protozoa_rust && cargo test test_draw_dashboard_uses_sidebar -- --nocapture`

Expected: FAIL (Agent panel not on right side yet)

**Step 3: Update implementation**

Replace `draw_dashboard` function:

```rust
/// Draws the full cognitive dashboard with sidebar layout.
pub fn draw_dashboard(f: &mut Frame, grid_lines: Vec<String>, state: &DashboardState) {
    let (main_area, sidebar) = compute_sidebar_layout(f.area());

    // === Left: Petri Dish (full height) ===
    draw_petri_dish_panel(f, main_area, grid_lines);

    // === Right Sidebar ===
    // [0] Metrics (top)
    draw_metrics_panel(f, sidebar[0], state);

    // [1] MCTS Planning
    draw_mcts_panel(f, sidebar[1], state);

    // [2] Landmarks
    draw_landmarks_panel(f, sidebar[2], state);

    // [3] Spatial Memory (bottom, takes remaining space)
    draw_spatial_grid_panel(f, sidebar[3], state);
}
```

**Step 4: Run test to verify it passes**

Run: `cd protozoa_rust && cargo test test_draw_dashboard_uses_sidebar -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
cd protozoa_rust && git add src/ui/render.rs && git commit -m "feat(ui): switch draw_dashboard to sidebar layout"
```

---

## Task 7: Run Full Test Suite and Fix Any Failures

**Files:**
- Modify: `protozoa_rust/src/ui/render.rs` (tests module)

**Step 1: Run all tests**

Run: `cd protozoa_rust && cargo test`

**Step 2: Fix any failing tests**

Common fixes needed:
- Update `test_boundary_coordinates` if it references old layout
- Remove or update any tests that depend on `compute_quadrant_layout`

**Step 3: Run clippy**

Run: `cd protozoa_rust && cargo clippy -- -D warnings`

Fix any warnings.

**Step 4: Run fmt**

Run: `cd protozoa_rust && cargo fmt`

**Step 5: Final test run**

Run: `cd protozoa_rust && cargo test && cargo clippy -- -D warnings`

Expected: All pass, no warnings

**Step 6: Commit**

```bash
cd protozoa_rust && git add -A && git commit -m "test(ui): update tests for sidebar layout"
```

---

## Task 8: Manual Visual Verification

**Step 1: Run the simulation**

Run: `cd protozoa_rust && cargo run --release`

**Step 2: Verify layout**

Check:
- [ ] Petri Dish takes ~70% width on left
- [ ] Sidebar is on right with 4 stacked panels
- [ ] No overlay in Petri Dish
- [ ] Metrics panel shows energy bar, mode, etc.
- [ ] MCTS panel shows planning info
- [ ] Landmarks panel shows table
- [ ] Spatial Memory grid fits and shows agent position
- [ ] No visual glitches or panics

**Step 3: Test narrow terminal**

Resize terminal to ~80x24 and verify:
- [ ] Layout still works
- [ ] Spatial grid compresses properly
- [ ] No panics

**Step 4: Commit if any final fixes needed**

```bash
cd protozoa_rust && git add -A && git commit -m "fix(ui): final layout adjustments"
```

---

## Summary

| Task | Description | Estimated Steps |
|------|-------------|-----------------|
| 1 | Add `compute_sidebar_layout` | 5 |
| 2 | Add `draw_metrics_panel` | 5 |
| 3 | Add `compress_spatial_grid` | 5 |
| 4 | Update spatial panel compression | 5 |
| 5 | Simplify petri dish panel | 4 |
| 6 | Update `draw_dashboard` | 5 |
| 7 | Fix tests and lint | 6 |
| 8 | Manual verification | 4 |

**Total: 8 tasks, ~39 steps**
