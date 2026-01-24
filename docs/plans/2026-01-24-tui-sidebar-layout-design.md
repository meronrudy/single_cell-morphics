# TUI Sidebar Layout Optimization

## Problem

The current 2x2 quadrant layout is unbalanced:
- **Petri Dish (top-left):** Overloaded with ASCII visualization + metrics overlay
- **Other panels:** Mostly empty space (Spatial Memory 20x10 grid, MCTS 7 lines, Landmarks 10 lines)

## Goals

1. Maximize Petri Dish visualization area
2. Better information density across all panels

## Design

### Layout Structure

Change from 2x2 quadrants to main + sidebar:

```
┌──────────────────────────────┬─────────────┐
│                              │  Metrics    │
│                              │  (8 lines)  │
│                              ├─────────────┤
│        Petri Dish            │  MCTS       │
│        (~70% width)          │  (9 lines)  │
│                              ├─────────────┤
│        Full height           │  Landmarks  │
│        No overlay            │  (12 lines) │
│                              ├─────────────┤
│                              │  Spatial    │
│                              │  (remaining)│
└──────────────────────────────┴─────────────┘
```

- **Horizontal split:** 70% Petri Dish / 30% Sidebar
- **Sidebar vertical split:** Proportional to content
  - Metrics: `Constraint::Length(8)`
  - MCTS: `Constraint::Length(9)`
  - Landmarks: `Constraint::Length(12)`
  - Spatial Memory: `Constraint::Min(0)` (takes remaining)

### Petri Dish Panel

- Removes metrics overlay entirely (moved to sidebar)
- Uses full inner area for ASCII visualization
- Gains ~23 columns previously used by overlay

### Metrics Panel (new)

Extracts overlay content into dedicated sidebar panel:

```
┌─ Agent ─────────────┐
│ E:[████████░░] 80%  │
│ Mode: EXPLORING     │
│ PE: -0.23  ρ:34.60  │
│ v: 0.3  θ: 231°     │
│ L:0.62  R:0.51      │
│ ∂t: 0.03            │
└─────────────────────┘
```

### MCTS Panel

Same content, relocated to sidebar:

```
┌─ MCTS Planning ─────┐
│ Best: ↗ (L)         │
│ G: 15.76            │
│ ├─Prag: 5.12        │
│ └─Epis: 35.48       │
│ Rolls: 50           │
│ Depth: 10           │
│ Replan: 19          │
└─────────────────────┘
```

### Landmarks Panel

Same table format:

```
┌─ Landmarks ─────────┐
│ # │ Pos    │Rel│Vis │
│───┼────────┼───┼────│
│ 1 │(45,23) │.92│ 3  │
│ 2 │  --    │-- │ -  │
│ ...                 │
└─────────────────────┘
```

### Spatial Memory Compression

The 20x10 grid must fit in ~25 character sidebar width.

**Dynamic compression:**
- At render time, check `inner.width` of Spatial panel
- If `available_width < 20`: compress horizontally
- Compression ratio: `original_width / available_width`
- Merge adjacent cells using weighted average of means
- Agent position marker (`○`) placed at compressed coordinate
- Height (10 rows) preserved

**New function:**
```rust
fn compress_spatial_grid(
    cells: &[CellPrior],
    orig_width: usize,
    orig_height: usize,
    target_width: usize,
) -> Vec<CellPrior>
```

## Implementation

### Files to Modify

**`src/ui/render.rs`:**
- Replace `compute_quadrant_layout` with `compute_sidebar_layout`
- Update `draw_dashboard` to use new layout
- Remove overlay rendering from `draw_petri_dish_panel`
- Add `draw_metrics_panel` function
- Add `compress_spatial_grid` function
- Update `draw_spatial_grid_panel` to use compression

### No Changes Needed

- `field.rs` - grid computation unchanged
- `mod.rs` - `DashboardState` struct unchanged
- `params.rs` - no parameter changes
- `main.rs` - calls `draw_dashboard` unchanged

### Testing

- Update tests referencing `compute_quadrant_layout`
- Add test for `compress_spatial_grid`
- Existing formatting tests remain valid

## Estimated Scope

- ~50 lines modified in layout logic
- ~30 lines new for compression function
- ~20 lines for new metrics panel function
- Test updates
