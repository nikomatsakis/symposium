import AVFoundation
import AppKit
import ApplicationServices
import Foundation
import ScreenCaptureKit

class PermissionManager: ObservableObject {
    @Published var hasAccessibilityPermission = false
    @Published var hasScreenRecordingPermission = false

    func checkAllPermissions() -> Bool {
        checkAccessibilityPermission()
        checkScreenRecordingPermission()
        return hasAccessibilityPermission && hasScreenRecordingPermission
    }

    func checkAccessibilityPermission() {
        let options: [String: Any] = [
            kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: false
        ]
        hasAccessibilityPermission = AXIsProcessTrustedWithOptions(options as CFDictionary)
    }

    func requestAccessibilityPermission() {
        let options: [String: Any] = [
            kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true
        ]
        _ = AXIsProcessTrustedWithOptions(options as CFDictionary)
    }

    func checkScreenRecordingPermission() {
        // Use ScreenCaptureKit for macOS 12.3+
        if #available(macOS 12.3, *) {
            Task {
                do {
                    let content = try await SCShareableContent.current
                    let hasPermission = !content.displays.isEmpty
                    await MainActor.run {
                        hasScreenRecordingPermission = hasPermission
                    }
                } catch {
                    await MainActor.run {
                        hasScreenRecordingPermission = false
                    }
                }
            }
        } else {
            // For older versions, assume permission is granted (fallback)
            hasScreenRecordingPermission = true
        }
    }

    func requestScreenRecordingPermission() {
        // Use ScreenCaptureKit for macOS 12.3+ to trigger permission dialog
        if #available(macOS 12.3, *) {
            Task {
                do {
                    _ = try await SCShareableContent.current
                } catch {
                    // Error will occur if no permission, which is expected
                }
            }
        }
    }

    func openSystemPreferences(for permission: PermissionType) {
        switch permission {
        case .accessibility:
            if let url = URL(
                string:
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            {
                NSWorkspace.shared.open(url)
            }
        case .screenRecording:
            if let url = URL(
                string:
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            {
                NSWorkspace.shared.open(url)
            }
        }
    }
}

enum PermissionType {
    case accessibility
    case screenRecording
}
