import Foundation
import AppKit

/// Tracks and positions windows in a tiled grid layout
struct WindowTileTracker {
    private let tileManager: WindowTileManager
    private var visibleTaskspaceManager: VisibleTaskspaceManager
    
    init(taskspaces: [UUID]) {
        self.tileManager = WindowTileManager()
        self.visibleTaskspaceManager = VisibleTaskspaceManager()
        // Initialize with taskspaces
        for taskspaceId in taskspaces {
            self.visibleTaskspaceManager.addTaskspace(taskspaceId)
        }
    }
    
    /// Add a taskspace to the visible list and reposition grid
    mutating func addTaskspace(_ taskspaceId: UUID) {
        visibleTaskspaceManager.addTaskspace(taskspaceId)
    }
    
    /// Activate a taskspace and reposition all windows in grid
    mutating func activateTaskspace(_ taskspaceId: UUID, windowMappings: [UUID: CGWindowID], panelWidth: CGFloat) -> Bool {
        visibleTaskspaceManager.activateTaskspace(taskspaceId)
        return positionWindowsInGrid(windowMappings: windowMappings, panelWidth: panelWidth)
    }
    
    /// Position all visible windows in calculated grid layout
    private func positionWindowsInGrid(windowMappings: [UUID: CGWindowID], panelWidth: CGFloat) -> Bool {
        let visibleTaskspaces = visibleTaskspaceManager.visible
        
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
        
        // Calculate grid layout
        let gridRects = tileManager.calculateTileLayout(
            visibleTaskspaces: visibleTaskspaces.count,
            availableArea: availableArea
        )
        
        // Position each visible window
        var successCount = 0
        for (index, taskspaceId) in visibleTaskspaces.enumerated() {
            guard index < gridRects.count,
                  let windowID = windowMappings[taskspaceId] else { continue }
            
            let rect = gridRects[index]
            if WindowPositioner.positionWindow(windowID: windowID, at: rect) {
                successCount += 1
            }
        }
        
        return successCount > 0
    }
    
    /// Get current visible taskspaces
    func getVisibleTaskspaces() -> [UUID] {
        return visibleTaskspaceManager.visible
    }
    
    /// Get count of visible taskspaces
    func getVisibleCount() -> Int {
        return visibleTaskspaceManager.visible.count
    }
}
