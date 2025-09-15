import Foundation
import AppKit

/// Utility for positioning and resizing windows using macOS Accessibility APIs
struct WindowPositioner {
    
    /// Get window element for accessibility operations
    static func getWindowElement(for windowID: CGWindowID) -> AXUIElement? {
        // Get window info to find the owning process
        let options = CGWindowListOption(arrayLiteral: .excludeDesktopElements)
        guard let windowList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
            return nil
        }
        
        guard let windowInfo = windowList.first(where: { window in
            if let id = window[kCGWindowNumber as String] as? CGWindowID {
                return id == windowID
            }
            return false
        }) else { return nil }
        
        guard let processID = windowInfo[kCGWindowOwnerPID as String] as? pid_t else { return nil }
        
        let app = AXUIElementCreateApplication(processID)
        
        var windowsRef: CFTypeRef?
        let result = AXUIElementCopyAttributeValue(app, kAXWindowsAttribute as CFString, &windowsRef)
        
        guard result == .success,
              let windows = windowsRef as? [AXUIElement] else {
            return nil
        }
        
        // Find the window with matching CGWindowID
        for window in windows {
            if let axWindowID = getWindowID(from: window), axWindowID == windowID {
                return window
            }
        }
        
        return nil
    }
    
    /// Position and resize window to specific bounds
    static func positionWindow(windowID: CGWindowID, at rect: CGRect) -> Bool {
        guard let windowElement = getWindowElement(for: windowID) else { 
            return false
        }
        
        // Set position
        var position = rect.origin
        let positionValue = AXValueCreate(.cgPoint, &position)!
        let positionResult = AXUIElementSetAttributeValue(windowElement, kAXPositionAttribute as CFString, positionValue)
        
        // Set size
        var size = rect.size
        let sizeValue = AXValueCreate(.cgSize, &size)!
        let sizeResult = AXUIElementSetAttributeValue(windowElement, kAXSizeAttribute as CFString, sizeValue)
        
        return positionResult == .success && sizeResult == .success
    }
    
    /// Move window by relative offset
    static func moveWindow(windowID: CGWindowID, by delta: CGPoint) -> Bool {
        guard let windowElement = getWindowElement(for: windowID) else { return false }
        
        // Get current position
        var positionRef: CFTypeRef?
        guard AXUIElementCopyAttributeValue(windowElement, kAXPositionAttribute as CFString, &positionRef) == .success,
              let positionValue = positionRef else { return false }
        
        var currentPos = CGPoint.zero
        AXValueGetValue(positionValue as! AXValue, .cgPoint, &currentPos)
        
        // Calculate new position
        var newPos = CGPoint(x: currentPos.x + delta.x, y: currentPos.y + delta.y)
        let newPosValue = AXValueCreate(.cgPoint, &newPos)!
        
        // Set new position
        let result = AXUIElementSetAttributeValue(windowElement, kAXPositionAttribute as CFString, newPosValue)
        return result == .success
    }
    
    /// Resize window to specific size
    static func resizeWindow(windowID: CGWindowID, to size: CGSize) -> Bool {
        guard let windowElement = getWindowElement(for: windowID) else { return false }
        
        var sizeValue = size
        let axSizeValue = AXValueCreate(.cgSize, &sizeValue)!
        let result = AXUIElementSetAttributeValue(windowElement, kAXSizeAttribute as CFString, axSizeValue)
        
        return result == .success
    }
}
