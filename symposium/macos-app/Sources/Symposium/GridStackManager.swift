import Foundation
import AppKit

/// Manages a dynamic grid of window stacks for tile mode
///
/// This struct implements a flexible tiling system where:
/// - Grid positions grow dynamically from 0 to maxGridPositions (currently 4)
/// - Each grid position can stack multiple windows (back-to-front order)
/// - Layout adapts based on position count: 1→full screen, 2→split, 3→L-shape, 4→2x2
/// - New windows create positions until max, then stack in least crowded position
/// - Empty positions are automatically cleaned up when windows close
/// - Only the front window of each stack is visible and positioned
///
/// Example: With 3 windows, you get 3 grid positions each with 1 window.
/// With 6 windows at max=4, you get 4 positions with 2 windows stacked in some positions.
struct GridStackManager {
    private var gridPositions: [[UUID]] = []  // Each array is a stack (back-to-front)
    private let maxGridPositions: Int = 4
    
    /// Add a window to the grid, creating new position if needed
    mutating func addWindow(_ windowId: UUID) {
        // If we haven't reached max positions, create a new one
        if gridPositions.count < maxGridPositions {
            gridPositions.append([windowId])
        } else {
            // Find the position with fewest windows and add there
            let leastCrowdedIndex = gridPositions.enumerated().min { $0.element.count < $1.element.count }?.offset ?? 0
            gridPositions[leastCrowdedIndex].append(windowId)
        }
    }
    
    /// Remove a window from the grid, cleaning up empty positions
    mutating func removeWindow(_ windowId: UUID) {
        for i in 0..<gridPositions.count {
            if let index = gridPositions[i].firstIndex(of: windowId) {
                gridPositions[i].remove(at: index)
                
                // If position is now empty, try to redistribute before removing
                if gridPositions[i].isEmpty {
                    // Try to steal a window from another position to keep this position alive
                    if let donorIndex = findDonorPosition() {
                        // Take the second-to-top window (preserve front window visibility)
                        let donorStack = gridPositions[donorIndex]
                        if donorStack.count > 1 {
                            let stolenWindow = gridPositions[donorIndex].remove(at: donorStack.count - 2)
                            gridPositions[i].append(stolenWindow)
                        } else {
                            // No suitable donor, remove empty position
                            gridPositions.remove(at: i)
                        }
                    } else {
                        // No donors available, remove empty position
                        gridPositions.remove(at: i)
                    }
                }
                break
            }
        }
    }
    
    /// Find a position with multiple windows that can donate one
    private func findDonorPosition() -> Int? {
        // Find position with most windows (and at least 2)
        return gridPositions.enumerated()
            .filter { $0.element.count > 1 }
            .max { $0.element.count < $1.element.count }?
            .offset
    }
    
    /// Activate a window, bringing it to the front of its stack
    mutating func activateWindow(_ windowId: UUID) {
        for i in 0..<gridPositions.count {
            if let index = gridPositions[i].firstIndex(of: windowId) {
                // Move to front of stack (last position = front)
                let window = gridPositions[i].remove(at: index)
                gridPositions[i].append(window)
                break
            }
        }
    }
    
    /// Get all windows that should be visible (front of each stack)
    func getVisibleWindows() -> [UUID] {
        return gridPositions.compactMap { $0.last }
    }
    
    /// Get all windows in a specific grid position (back-to-front order)
    func getWindowStack(at position: Int) -> [UUID] {
        guard position < gridPositions.count else { return [] }
        return gridPositions[position]
    }
    
    /// Get the front window for a specific grid position
    func getFrontWindow(at position: Int) -> UUID? {
        guard position < gridPositions.count else { return nil }
        return gridPositions[position].last
    }
    
    /// Calculate grid layout bounds for current number of positions
    func calculateGridBounds(availableArea: CGRect) -> [CGRect] {
        let positionCount = gridPositions.count
        guard positionCount > 0 else { return [] }
        
        switch positionCount {
        case 1:
            // Single window takes full area
            return [availableArea]
            
        case 2:
            // Left/right split
            let width = availableArea.width / 2
            return [
                CGRect(x: availableArea.minX, y: availableArea.minY, width: width, height: availableArea.height),
                CGRect(x: availableArea.minX + width, y: availableArea.minY, width: width, height: availableArea.height)
            ]
            
        case 3:
            // L-shape: one large on left, two stacked on right
            let leftWidth = availableArea.width * 0.6
            let rightWidth = availableArea.width * 0.4
            let rightHeight = availableArea.height / 2
            
            return [
                CGRect(x: availableArea.minX, y: availableArea.minY, width: leftWidth, height: availableArea.height),
                CGRect(x: availableArea.minX + leftWidth, y: availableArea.minY, width: rightWidth, height: rightHeight),
                CGRect(x: availableArea.minX + leftWidth, y: availableArea.minY + rightHeight, width: rightWidth, height: rightHeight)
            ]
            
        case 4:
            // 2x2 grid
            let width = availableArea.width / 2
            let height = availableArea.height / 2
            
            return [
                CGRect(x: availableArea.minX, y: availableArea.minY, width: width, height: height),
                CGRect(x: availableArea.minX + width, y: availableArea.minY, width: width, height: height),
                CGRect(x: availableArea.minX, y: availableArea.minY + height, width: width, height: height),
                CGRect(x: availableArea.minX + width, y: availableArea.minY + height, width: width, height: height)
            ]
            
        default:
            // Fallback: distribute evenly in rows
            let columns = min(positionCount, 3)
            let rows = (positionCount + columns - 1) / columns
            let cellWidth = availableArea.width / CGFloat(columns)
            let cellHeight = availableArea.height / CGFloat(rows)
            
            var bounds: [CGRect] = []
            for i in 0..<positionCount {
                let col = i % columns
                let row = i / columns
                let x = availableArea.minX + CGFloat(col) * cellWidth
                let y = availableArea.minY + CGFloat(row) * cellHeight
                bounds.append(CGRect(x: x, y: y, width: cellWidth, height: cellHeight))
            }
            return bounds
        }
    }
    
    /// Get current number of grid positions
    var positionCount: Int {
        return gridPositions.count
    }
    
    /// Check if a window exists in the grid
    func containsWindow(_ windowId: UUID) -> Bool {
        return gridPositions.contains { $0.contains(windowId) }
    }
    
    /// Get total number of windows across all positions
    var totalWindowCount: Int {
        return gridPositions.reduce(0) { $0 + $1.count }
    }
}
