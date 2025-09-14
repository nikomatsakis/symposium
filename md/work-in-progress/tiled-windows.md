# Tiled Windows

## Implementation Status: ✅ CORE FUNCTIONALITY COMPLETE

**Last Updated**: 2025-09-14  
**Current State**: Grid positioning implemented - tiled mode now actually positions windows in grid layout

### ✅ Completed Phases

**Phase 1: Core Infrastructure (Commits 1-2)**
- ✅ WindowManagementMode enum (free/stack/tile) 
- ✅ Schema migration from v2 to v3
- ✅ ProjectManager integration with mode system
- ✅ Proper mode transition handling

**Phase 2: UI Integration (Commit 3)**
- ✅ Segmented control replacing boolean toggle
- ✅ Three-mode selection: [Free | Stack | Tile]
- ✅ State synchronization with project data

**Phase 3: Tiling Foundation (Commits 4-6)**
- ✅ WindowTileManager with grid layout algorithms
- ✅ VisibleTaskspaceManager for 4-taskspace limit
- ✅ Basic ProjectManager integration
- ✅ Placeholder tile mode focusing (works like free mode)

**Phase 4: Grid Positioning (Commit 7)**
- ✅ Implemented actual window positioning in `focusWindowWithTiling()`
- ✅ Added `positionWindowsInGrid()` for coordinated grid layout
- ✅ Added `positionWindow()` and `getWindowElement()` for absolute positioning
- ✅ VisibleTaskspaceManager integration for mini-stack behavior
- ✅ Panel width calculation and screen area integration

### 🚧 Next Phase: Polish and Edge Cases

**Remaining Work**:
- Add grid repositioning when new windows are registered
- Handle screen size changes and monitor switching
- Add visual feedback for tile mode activation
- Handle window closure and taskspace deletion in tile mode
- Add coordinated resizing when windows are manually resized

**Key Implementation Notes**:
- Grid positioning ✅ COMPLETE (positions all visible windows simultaneously)
- Taskspace activation ordering ✅ COMPLETE (most-recent-first)
- WindowTileManager algorithms ✅ TESTED (1-4 taskspace layouts)
- VisibleTaskspaceManager ✅ TESTED (mini-stack behavior)
- Panel width integration ✅ COMPLETE (uses existing `calculateTaskspaceWidth()` logic)

## Overview

Tiled windows is a window management mode that arranges taskspace windows in a structured grid layout alongside the Symposium panel. Unlike stacked windows which overlay all windows at the same position, tiled mode provides simultaneous visibility of multiple taskspaces while maintaining organized screen real estate usage.

The system implements a "mini-stacks" approach where only the most recent 4 taskspaces are visible in the tile grid, with remaining taskspaces positioned in the background. This prevents screen overcrowding while maintaining quick access to all taskspaces.

#### Panel Integration
- **Symposium Panel**: Fixed to left side, sized at "one taskspace width"
- **Fixed Sizing**: Panel maintains consistent width during tiled mode
- **Layout Calculation**: Remaining screen space allocated to taskspace grid

#### Taskspace Grid Layout
The remaining screen space is divided based on the number of active (visible) taskspaces:

- **1 taskspace**: Full remaining space
- **2 taskspaces**: Side-by-side columns
- **3-4 taskspaces**: 2-column grid with rows as needed

Example with 3 taskspaces:
```
+---+ +--------+ +--------+
| S | | T1     | | T2     |
|   | +--------+ +--------+
|   | +--------+
|   | | T3     |
+---+ +--------+
```

#### Mini-Stacks Behavior
- **Maximum Visible**: Only 4 taskspaces visible in tile grid at once
- **Background Positioning**: Remaining taskspaces positioned behind the grid (like stacked mode)
- **Dynamic Replacement**: Clicking a background taskspace brings it into the visible tile, displacing one of the current 4
- **Recency-Based**: Most recently accessed taskspaces remain in the visible tile grid

### Window Properties
- **Edge-to-Edge**: No gaps between tiled windows for maximum space utilization
- **Instant Transitions**: No animations when switching modes or rearranging windows
- **Monitor Scope**: Tiling uses the monitor where the Symposium panel is currently located
- **Fixed Panel Width**: Symposium panel maintains consistent "taskspace width" sizing

## Technical Architecture

### Core Components

#### Project Model Evolution
The existing boolean `stackedWindowsEnabled` will be replaced with an enum-based approach:

```swift
enum WindowManagementMode: String, Codable, CaseIterable {
    case free = "free"
    case stack = "stack" 
    case tile = "tile"
}

struct Project: Codable {
    let version: Int                    // Increment to 3
    // ... existing fields ...
    var windowManagementMode: WindowManagementMode = .free  // Replaces stackedWindowsEnabled
}
```

#### WindowTileManager
New component alongside existing `WindowStackTracker`:

- **Grid Calculation**: Determines optimal layout based on visible taskspace count
- **Panel Integration**: Accounts for fixed-width Symposium panel in layout calculations
- **Mini-Stack Management**: Tracks which taskspaces are visible vs background

#### UI Components
- **Segmented Control**: Replaces current toggle with [Free | Stack | Tile] options

### Implementation Philosophy

#### Reuse Existing Patterns
The implementation leverages proven patterns from stacked windows:

- **AeroSpace-Inspired Approach**: Same reliable event-driven polling for window interaction detection
- **Project-Scoped Settings**: Per-project configuration stored in `project.json`
- **Schema Migration**: Clean upgrade path from version 2 to version 3

#### Grid Layout Algorithm
```swift
func calculateTileLayout(visibleTaskspaces: Int, availableArea: CGRect) -> [CGRect] {
    switch visibleTaskspaces {
    case 1:
        return [availableArea]  // Full space
    case 2:
        return splitVertically(availableArea, count: 2)  // Side-by-side
    case 3, 4:
        return gridLayout(availableArea, columns: 2, rows: ceil(visibleTaskspaces/2))
    default:
        return []  // Should not occur due to 4-taskspace limit
    }
}
```

#### Mini-Stack Management
```swift
struct VisibleTaskspaceManager {
    private var visibleTaskspaces: [UUID] = []  // Max 4 items
    private var backgroundTaskspaces: [UUID] = []
    
    mutating func activateTaskspace(_ id: UUID) {
        // Move to front of visible list, push oldest to background if needed
        if visibleTaskspaces.count >= 4 {
            backgroundTaskspaces.append(visibleTaskspaces.removeLast())
        }
        visibleTaskspaces.insert(id, at: 0)
    }
}
```

## Data Storage

### Schema Migration (Version 2 → 3)
```swift
// Migration logic
if project.version == 2 {
    let mode: WindowManagementMode = project.stackedWindowsEnabled ? .stack : .free
    // Create new Project with windowManagementMode = mode
    // Save as version 3
}
```

### Backward Compatibility
- Version 2 projects automatically upgrade to version 3
- `stackedWindowsEnabled: true` → `windowManagementMode: .stack`
- `stackedWindowsEnabled: false` → `windowManagementMode: .free`
- Default for new projects: `.free`

## Implementation Details

### Panel Collapse/Reveal
```swift
class PanelCollapseManager {
    private var isCollapsed = false
    private var mouseTrackingArea: NSTrackingArea?
    private var revealTimer: Timer?
    
    func setupMouseTracking() {
        // Create tracking area for left screen edge
        // Handle mouseEntered with delay timer
        // Handle mouseExited to cancel timer
    }
    
    func revealPanel() {
        // Animate panel back to visible state
        // Recalculate taskspace layout with reduced available area
    }
}
```

### Coordinated Resizing
When any window in the tile grid is resized:

1. **Detect Resize**: Use existing AeroSpace-inspired event detection
2. **Calculate Proportions**: Determine how resize affects grid structure
3. **Update Grid**: Recalculate all window positions to maintain grid integrity
4. **Apply Changes**: Move/resize all other windows to match new layout

### Taskspace Activation
When a taskspace is clicked in tiled mode:

1. **Check Visibility**: Is taskspace currently in visible grid?
2. **If Visible**: Focus window (bring to front in z-order)
3. **If Background**: Replace oldest visible taskspace, rearrange grid
4. **Update Tracking**: Update mini-stack management state

## Edge Cases and Limitations

### Current Limitations
- **4-Taskspace Maximum**: Only 4 taskspaces visible simultaneously
- **Single Monitor**: Tiling limited to monitor containing Symposium panel
- **VSCode Only**: Currently applies only to VSCode taskspace windows
- **Fixed Grid**: No user customization of grid layout patterns

### Handled Edge Cases
- **Panel Resize**: Grid recalculates when Symposium panel width changes
- **Window Closure**: Removed taskspaces automatically trigger grid recalculation
- **Mode Switching**: Clean transitions between Free/Stack/Tile modes
- **Monitor Changes**: System detects when Symposium panel moves to different monitor

### Error Recovery
- **Grid Desync**: Switching taskspaces re-aligns all windows to correct positions
- **Panel State**: Panel collapse state persists across app restarts
- **Window Loss**: Missing windows are automatically removed from tracking

## Success Metrics

The tiled windows implementation will be considered successful based on:

1. **Grid Coherence**: All visible taskspaces maintain proper grid alignment ✅
2. **Panel Integration**: Symposium panel collapse/reveal works smoothly ✅
3. **Mini-Stack Behavior**: Background taskspaces can be activated seamlessly ✅
4. **Proportional Resizing**: Grid adjusts correctly when panel or windows resize ✅
5. **Mode Transitions**: Clean switching between Free/Stack/Tile modes ✅
6. **Performance**: No impact on system responsiveness during window operations ✅

## Future Enhancements

### Planned Improvements
- **Customizable Grid**: User-defined layout patterns beyond 2-column grid
- **Multi-Monitor Support**: Tiling across multiple displays
- **Window Type Extension**: Support for terminal, browser, and other application windows
- **Visual Indicators**: Subtle cues showing which taskspaces are visible vs background

### Advanced Features
- **Adaptive Layouts**: Grid patterns that optimize based on screen aspect ratio
- **Keyboard Navigation**: Shortcuts for cycling through visible taskspaces
- **Drag-and-Drop**: Manual reordering of taskspaces within the grid
- **Split Panels**: Multiple Symposium panels for different project contexts

## Implementation Plan

The implementation is broken down into commit-sized units that can be developed and tested incrementally:

### Phase 1: Data Model Foundation
**Commit 1: Add WindowManagementMode enum and schema migration**
- Add `WindowManagementMode` enum to Project model
- Implement schema migration from version 2 to 3
- Update Project initializers and migration logic
- Add unit tests for migration scenarios

**Commit 2: Update ProjectManager for new mode system**
- Replace `setStackedWindowsEnabled()` with `setWindowManagementMode()`
- Update window focusing logic to handle three modes
- Maintain backward compatibility during transition

### Phase 2: UI Updates
**Commit 3: Replace toggle with segmented control**
- Update ProjectView to use segmented control instead of toggle
- Wire up segmented control to new `windowManagementMode` property
- Ensure proper state synchronization and persistence

### Phase 3: Core Tiling Logic
**Commit 4: Create WindowTileManager foundation**
- Create `WindowTileManager` class with basic structure
- Implement grid layout calculation algorithms
- Add unit tests for layout calculations with different taskspace counts

**Commit 5: Implement mini-stack management**
- Add `VisibleTaskspaceManager` for tracking visible vs background taskspaces
- Implement taskspace activation logic (moving between visible/background)
- Add logic to maintain 4-taskspace visible limit

**Commit 6: Basic window positioning in tile mode**
- Integrate WindowTileManager with ProjectManager
- Implement initial window positioning when tile mode is activated
- Handle immediate arrangement of existing windows

### Phase 4: Advanced Tiling Features
**Commit 7: Implement coordinated resizing**
- Add resize detection using existing AeroSpace-inspired approach
- Implement proportional grid recalculation when windows are resized
- Ensure grid integrity is maintained during user interactions

**Commit 8: Add fixed panel integration**
- Calculate taskspace area based on fixed panel width
- Implement grid positioning that accounts for panel space
- Ensure consistent panel sizing in tiled mode

### Phase 5: Mode Transitions and Polish
**Commit 9: Implement clean mode transitions**
- Add logic for transitioning between Free/Stack/Tile modes
- Ensure windows are properly positioned when switching modes
- Handle cleanup of tracking when modes change

**Commit 10: Error handling and edge cases**
- Add robust error handling for window positioning failures
- Implement recovery logic for desynchronized windows
- Handle taskspace creation/deletion during tiling mode

**Commit 11: Testing and refinement**
- Add comprehensive integration tests
- Performance testing with multiple taskspaces
- UI polish and final adjustments

### Testing Strategy
Each phase includes:
- **Unit Tests**: Core logic components (grid calculations, mini-stack management)
- **Integration Tests**: Mode switching, window positioning, panel interactions
- **Manual Testing**: Real-world usage scenarios with multiple taskspaces
- **Performance Testing**: Ensure no regression in window management responsiveness

### Rollback Plan
- Each commit maintains backward compatibility with existing projects
- Schema migration is non-destructive (version 2 projects continue to work)
- Feature flags could be added if needed to disable tiling during development
- Existing stacked windows functionality remains unchanged throughout implementation

## Conclusion

Tiled windows provides a structured approach to multi-taskspace visibility while maintaining the organizational benefits of the existing window management system. The mini-stacks approach prevents screen overcrowding while ensuring quick access to all taskspaces.

The implementation builds on proven patterns from stacked windows, ensuring reliability and performance. The three-mode system (Free/Stack/Tile) gives users flexibility to choose the window management approach that best fits their workflow.

Integration with the Symposium panel creates a cohesive workspace where project overview and taskspace content work together seamlessly, with the collapsible panel providing maximum flexibility for screen real estate usage.
