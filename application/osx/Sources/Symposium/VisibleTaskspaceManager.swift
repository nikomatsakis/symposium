import Foundation

/// Manages which taskspaces are visible in the tile grid vs background (mini-stacks approach)
struct VisibleTaskspaceManager {
    private var visibleTaskspaces: [UUID] = []  // Max 4 items, most recent first
    private var backgroundTaskspaces: [UUID] = []
    
    private let maxVisibleCount = 4
    
    /// Get currently visible taskspace IDs in order (most recent first)
    var visible: [UUID] {
        return visibleTaskspaces
    }
    
    /// Get background taskspace IDs
    var background: [UUID] {
        return backgroundTaskspaces
    }
    
    /// Check if a taskspace is currently visible in the tile grid
    func isVisible(_ taskspaceId: UUID) -> Bool {
        return visibleTaskspaces.contains(taskspaceId)
    }
    
    /// Activate a taskspace, bringing it to the front of the visible list
    mutating func activateTaskspace(_ taskspaceId: UUID) {
        // Remove from both lists first
        visibleTaskspaces.removeAll { $0 == taskspaceId }
        backgroundTaskspaces.removeAll { $0 == taskspaceId }
        
        // Add to front of visible list
        visibleTaskspaces.insert(taskspaceId, at: 0)
        
        // If we exceed the limit, move the oldest visible to background
        if visibleTaskspaces.count > maxVisibleCount {
            let oldestVisible = visibleTaskspaces.removeLast()
            backgroundTaskspaces.insert(oldestVisible, at: 0)
        }
    }
    
    /// Add a new taskspace (e.g., when created)
    mutating func addTaskspace(_ taskspaceId: UUID) {
        // New taskspaces are automatically activated (brought to front)
        activateTaskspace(taskspaceId)
    }
    
    /// Remove a taskspace (e.g., when deleted)
    mutating func removeTaskspace(_ taskspaceId: UUID) {
        visibleTaskspaces.removeAll { $0 == taskspaceId }
        backgroundTaskspaces.removeAll { $0 == taskspaceId }
    }
    
    /// Initialize with existing taskspaces (most recent first)
    mutating func initialize(with taskspaceIds: [UUID]) {
        visibleTaskspaces.removeAll()
        backgroundTaskspaces.removeAll()
        
        // Take first 4 as visible, rest as background
        let visible = Array(taskspaceIds.prefix(maxVisibleCount))
        let background = Array(taskspaceIds.dropFirst(maxVisibleCount))
        
        visibleTaskspaces = visible
        backgroundTaskspaces = background
    }
    
    /// Get total count of managed taskspaces
    var totalCount: Int {
        return visibleTaskspaces.count + backgroundTaskspaces.count
    }
}
