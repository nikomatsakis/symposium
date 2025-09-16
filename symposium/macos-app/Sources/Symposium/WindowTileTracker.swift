import Foundation
import AppKit

/// Tracks and positions windows in a tiled grid layout with stacked positions
struct WindowTileTracker {
    private let tileManager: WindowTileManager
    private var gridStackManager: GridStackManager
    
    init() {
        self.tileManager = WindowTileManager()
        self.gridStackManager = GridStackManager()
    }
    
    /// Add a taskspace window to the grid
    mutating func addWindow(_ windowId: UUID) {
        gridStackManager.addWindow(windowId)
    }
    
    /// Remove a taskspace window from the grid
    mutating func removeWindow(_ windowId: UUID) {
        gridStackManager.removeWindow(windowId)
    }
    
    /// Activate a taskspace and reposition all windows in grid
    mutating func activateWindow(_ windowId: UUID, windowMappings: [UUID: CGWindowID], panelWidth: CGFloat) -> Bool {
        gridStackManager.activateWindow(windowId)
        return positionWindowsInGrid(windowMappings: windowMappings, panelWidth: panelWidth)
    }
    
    /// Position all visible windows in calculated grid layout
    private func positionWindowsInGrid(windowMappings: [UUID: CGWindowID], panelWidth: CGFloat) -> Bool {
        // Get screen dimensions
        guard let screen = NSScreen.main else { return false }
        let screenFrame = screen.frame
        
        // Calculate available area (screen minus panel width)
        let availableArea = CGRect(
            x: panelWidth,
            y: screenFrame.minY,
            width: screenFrame.width - panelWidth,
            height: screenFrame.height
        )
        
        // Calculate grid layout bounds
        let gridBounds = gridStackManager.calculateGridBounds(availableArea: availableArea)
        
        // Position each visible window (front of each stack)
        var successCount = 0
        let visibleWindows = gridStackManager.getVisibleWindows()
        
        for (index, windowId) in visibleWindows.enumerated() {
            guard index < gridBounds.count,
                  let windowCGID = windowMappings[windowId] else { continue }
            
            let rect = gridBounds[index]
            if WindowPositioner.positionWindow(windowID: windowCGID, at: rect) {
                successCount += 1
            }
        }
        
        return successCount > 0
    }
    
    /// Get current visible windows (front of each stack)
    func getVisibleWindows() -> [UUID] {
        return gridStackManager.getVisibleWindows()
    }
    
    /// Get count of visible windows
    func getVisibleCount() -> Int {
        return gridStackManager.getVisibleWindows().count
    }
    
    /// Get total count of all windows in grid
    func getTotalCount() -> Int {
        return gridStackManager.totalWindowCount
    }
    
    /// Get number of grid positions
    func getPositionCount() -> Int {
        return gridStackManager.positionCount
    }
}
