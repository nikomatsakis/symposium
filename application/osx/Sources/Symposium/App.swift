import AppKit
import SwiftUI

@main
struct SymposiumApp: App {
    @StateObject private var appState = AppState()
    @StateObject private var agentManager = AgentManager()
    @StateObject private var settingsManager = SettingsManager()
    @StateObject private var permissionManager = PermissionManager()
    
    // App delegate for dock click handling
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    // SwiftUI environment for window management
    @Environment(\.openWindow) private var openWindow

    var body: some Scene {
        // Splash/Setup window - shows when no projects are open
        WindowGroup(id: "splash") {
            if appState.shouldShowSplash {
                SplashView()
                    .environmentObject(appState)
                    .environmentObject(agentManager)
                    .environmentObject(settingsManager)
                    .environmentObject(permissionManager)
                    .environmentObject(appDelegate)
                    .onAppear {
                        Logger.shared.log("Splash window appeared - no projects open")
                    }
            } else {
                // When we have projects, this window should not show content
                EmptyView()
                    .onAppear {
                        Logger.shared.log("Splash window hidden - projects are open")
                    }
            }
        }
        .windowResizability(.contentSize)
        .defaultAppStorage(.standard)
        
        // Project window - shows when we have an active project
        WindowGroup(id: "project") {
            if let firstProject = appState.openProjects.first {
                ProjectWindowView(project: firstProject)
                    .environmentObject(appState)
                    .environmentObject(agentManager)
                    .environmentObject(settingsManager)
                    .environmentObject(permissionManager)
                    .onAppear {
                        Logger.shared.log("Project window appeared: \(firstProject.name)")
                    }
                    .onDisappear {
                        Logger.shared.log("Project window disappeared: \(firstProject.name)")
                    }
            } else {
                EmptyView()
                    .onAppear {
                        Logger.shared.log("Project window hidden - no projects open")
                    }
            }
        }
        .windowResizability(.contentSize)
        .windowLevel(.floating) // Make project window float like the old NSPanel
        
        .commands {
            // File menu items
            CommandGroup(replacing: .newItem) {
                Button("New Project...") {
                    // Close all projects to show splash for project selection
                    Logger.shared.log("Menu: New Project - closing all projects to show splash")
                    appState.closeAllProjects()
                }
                .keyboardShortcut("n", modifiers: .command)
                
                Button("Open Project...") {
                    // Close all projects to show splash for project selection
                    Logger.shared.log("Menu: Open Project - closing all projects to show splash")
                    appState.closeAllProjects()
                }
                .keyboardShortcut("o", modifiers: .command)
            }
            
            CommandGroup(after: .help) {
                Button("Copy Debug Logs") {
                    copyLogsToClipboard()
                }
                .keyboardShortcut("d", modifiers: [.command, .shift])

                Button("List All Windows") {
                    listAllWindows()
                }
                .keyboardShortcut("w", modifiers: [.command, .shift])
                
                Divider()
                
                Button("Toggle Dock Panel") {
                    appDelegate.toggleDockPanel()
                }
                .keyboardShortcut("p", modifiers: [.command, .shift])
            }
        }

        Settings {
            SettingsView()
                .environmentObject(agentManager)
                .environmentObject(settingsManager)
                .environmentObject(permissionManager)
        }
    }


    private func copyLogsToClipboard() {
        let allLogs = Logger.shared.logs.joined(separator: "\n")
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(allLogs, forType: .string)
        Logger.shared.log("Copied \(Logger.shared.logs.count) log entries to clipboard")
    }

    private func listAllWindows() {
        Logger.shared.log("=== Window Enumeration Debug ===")

        // Get all windows using CGWindowListCopyWindowInfo
        let windowList =
            CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]]
            ?? []

        Logger.shared.log("Found \(windowList.count) total windows")

        for (index, window) in windowList.enumerated() {
            let windowID = window[kCGWindowNumber as String] as? CGWindowID ?? 0
            let ownerName = window[kCGWindowOwnerName as String] as? String ?? "Unknown"
            let windowName = window[kCGWindowName as String] as? String ?? "No Title"
            let layer = window[kCGWindowLayer as String] as? Int ?? 0

            // Only log windows that have titles or are from common apps
            if !windowName.isEmpty || ["Visual Studio Code", "VSCode", "Code"].contains(ownerName) {
                Logger.shared.log(
                    "[\(index)] ID:\(windowID) Owner:\(ownerName) Title:\"\(windowName)\" Layer:\(layer)"
                )
            }
        }

        Logger.shared.log("=== End Window List ===")

        // Also copy to clipboard for easy inspection
        var output = "=== Window Enumeration Debug ===\n"
        output += "Found \(windowList.count) total windows\n\n"

        for (index, window) in windowList.enumerated() {
            let windowID = window[kCGWindowNumber as String] as? CGWindowID ?? 0
            let ownerName = window[kCGWindowOwnerName as String] as? String ?? "Unknown"
            let windowName = window[kCGWindowName as String] as? String ?? "No Title"
            let layer = window[kCGWindowLayer as String] as? Int ?? 0

            if !windowName.isEmpty || ["Visual Studio Code", "VSCode", "Code"].contains(ownerName) {
                output +=
                    "[\(index)] ID:\(windowID) Owner:\(ownerName) Title:\"\(windowName)\" Layer:\(layer)\n"
            }
        }

        output += "\n=== End Window List ==="

        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(output, forType: .string)

        Logger.shared.log("Window list copied to clipboard")
    }
}
