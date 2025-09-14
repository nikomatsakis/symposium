import Foundation
import CoreGraphics

/// Manages window tiling layout and positioning for taskspace windows
class WindowTileManager {
    
    /// Calculate grid layout for given number of taskspaces
    func calculateTileLayout(visibleTaskspaces: Int, availableArea: CGRect) -> [CGRect] {
        switch visibleTaskspaces {
        case 0:
            return []
        case 1:
            return [availableArea]  // Full space
        case 2:
            return splitVertically(availableArea, count: 2)  // Side-by-side
        case 3, 4:
            let gridRects = gridLayout(availableArea, columns: 2, rows: Int(ceil(Double(visibleTaskspaces) / 2.0)))
            return Array(gridRects.prefix(visibleTaskspaces))  // Only return the number we need
        default:
            // Should not occur due to 4-taskspace limit in mini-stack management
            return []
        }
    }
    
    /// Split area vertically into equal columns
    private func splitVertically(_ area: CGRect, count: Int) -> [CGRect] {
        let width = area.width / CGFloat(count)
        var rects: [CGRect] = []
        
        for i in 0..<count {
            let x = area.origin.x + (CGFloat(i) * width)
            let rect = CGRect(x: x, y: area.origin.y, width: width, height: area.height)
            rects.append(rect)
        }
        
        return rects
    }
    
    /// Create grid layout with specified columns and rows
    private func gridLayout(_ area: CGRect, columns: Int, rows: Int) -> [CGRect] {
        let cellWidth = area.width / CGFloat(columns)
        let cellHeight = area.height / CGFloat(rows)
        var rects: [CGRect] = []
        
        for row in 0..<rows {
            for col in 0..<columns {
                let x = area.origin.x + (CGFloat(col) * cellWidth)
                let y = area.origin.y + (CGFloat(row) * cellHeight)
                let rect = CGRect(x: x, y: y, width: cellWidth, height: cellHeight)
                rects.append(rect)
            }
        }
        
        return rects
    }
    
    /// Calculate available area for taskspaces (screen minus panel width)
    /// panelWidth should be calculated using ProjectView.calculateTaskspaceWidth()
    func calculateTaskspaceArea(screenBounds: CGRect, panelWidth: CGFloat) -> CGRect {
        return CGRect(
            x: screenBounds.origin.x + panelWidth,
            y: screenBounds.origin.y,
            width: screenBounds.width - panelWidth,
            height: screenBounds.height
        )
    }
}
