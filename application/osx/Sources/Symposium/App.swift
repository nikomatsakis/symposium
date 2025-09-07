import AppKit
import SwiftUI

@main
struct SymposiumApp: App {
    // === State Machine Components ===
    @StateObject private var agentManager = AgentManager()
    @StateObject private var settingsManager = SettingsManager()
    @StateObject private var permissionManager = PermissionManager()
    
    // Simple state tracking (no reactive complexity)
    @State private var currentProject: Project? = nil
    
    // App delegate for dock click handling
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    // SwiftUI environment for explicit window management
    @Environment(\.openWindow) private var openWindow
    @Environment(\.dismissWindow) private var dismissWindow
    
    var body: some Scene {
        // === Three-Window State Machine ===
        // At any time, exactly ONE of these windows is open:
        // 1. Settings - missing permissions OR agent preference
        // 2. Project - have permissions + agent + current project  
        // 3. Splash - decision point / create-open project dialog
        
        // Splash/Setup window - project selection and creation
        WindowGroup(id: "splash") {
            SplashView(app: self)
                .environmentObject(agentManager)
                .environmentObject(settingsManager)
                .environmentObject(permissionManager)
                .environmentObject(appDelegate)
                .onAppear {
                    Logger.shared.log("Splash window appeared")
                }
        }
        .windowResizability(.contentSize)
        .defaultAppStorage(.standard)
        
        // Settings window - permissions and agent selection
        WindowGroup(id: "settings") {
            SettingsView()
                .environmentObject(agentManager)
                .environmentObject(settingsManager)
                .environmentObject(permissionManager)
                .onAppear {
                    Logger.shared.log("Settings window appeared")
                }
        }
        .windowResizability(.contentSize)
        
        // Project window - active project with panel interface
        WindowGroup(id: "project") {
            if let project = currentProject {
                ProjectWindowView(project: project)
                    .environmentObject(agentManager)
                    .environmentObject(settingsManager)
                    .environmentObject(permissionManager)
                    .onAppear {
                        Logger.shared.log("Project window appeared: \(project.name)")
                    }
                    .onDisappear {
                        Logger.shared.log("Project window disappeared: \(project.name)")
                    }
            } else {
                EmptyView()
                    .onAppear {
                        Logger.shared.log("Project window showed EmptyView - no current project")
                    }
            }
        }
        .windowResizability(.contentSize)
        .windowLevel(.floating) // Make project window float like the old NSPanel
        
        .commands {
            // File menu items
            CommandGroup(replacing: .newItem) {
                Button("New Project...") {
                    Logger.shared.log("Menu: New Project selected")
                    onNewProjectMenuSelected()
                }
                .keyboardShortcut("n", modifiers: .command)
                
                Button("Open Project...") {
                    Logger.shared.log("Menu: Open Project selected")
                    onOpenProjectMenuSelected()
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
                
                Button("Preferences...") {
                    Logger.shared.log("Menu: Preferences selected")
                    onPreferencesMenuSelected()
                }
                .keyboardShortcut(",", modifiers: .command)
            }
        }
    }
    
    // === Window State Machine ===
    // At startup and after any window closes, evaluate which window should be open
    
    /// Evaluates current state and opens the appropriate window
    /// Called at startup and whenever any window closes
    func evaluateWindowState() {
        Logger.shared.log("=== evaluateWindowState() ===")
        Logger.shared.log("Current state - hasAccessibility: \(permissionManager.hasAccessibilityPermission)")
        Logger.shared.log("Current state - hasScreenRecording: \(permissionManager.hasScreenRecordingPermission)")
        Logger.shared.log("Current state - agentScanCompleted: \(agentManager.scanningCompleted)")
        Logger.shared.log("Current state - currentProject: \(currentProject?.name ?? "nil")")
        
        dismissAllWindows() // Ensure clean state
        
        // Check permissions first
        if !permissionManager.hasAccessibilityPermission || !permissionManager.hasScreenRecordingPermission {
            Logger.shared.log("Missing permissions → Opening Settings window")
            openWindow(id: "settings")
            return
        }
        
        // Check if agent scanning is complete and we have a chosen agent
        if !agentManager.scanningCompleted {
            Logger.shared.log("Agent scanning not complete → Opening Splash window (will show scanning)")
            openWindow(id: "splash")
            return
        }
        
        // If we have a current project, show project window
        if let project = currentProject {
            Logger.shared.log("Have current project: \(project.name) → Opening Project window")
            openWindow(id: "project")
            return
        }
        
        // Try to load saved project
        if let savedProject = loadSavedProject() {
            Logger.shared.log("Restored saved project: \(savedProject.name) → Setting as current and opening Project window")
            currentProject = savedProject
            openWindow(id: "project")
            return
        }
        
        // Default: show splash for project selection
        Logger.shared.log("No current project → Opening Splash window (create/open project)")
        openWindow(id: "splash")
    }
    
    /// Load the saved project from settings if available and valid
    private func loadSavedProject() -> Project? {
        let savedPath = settingsManager.activeProjectPath
        Logger.shared.log("Checking saved project path: '\(savedPath)'")
        
        if savedPath.isEmpty {
            Logger.shared.log("No saved project path")
            return nil
        }
        
        // TODO: Actually load and validate the project
        // For now, return nil to force project selection
        Logger.shared.log("Project loading not implemented yet, returning nil")
        return nil
    }
    
    // === Explicit Window Management ===
    private func dismissAllWindows() {
        Logger.shared.log("Dismissing all windows for clean state")
        dismissWindow(id: "splash")
        dismissWindow(id: "settings") 
        dismissWindow(id: "project")
    }
    
    // === Callbacks from Windows ===
    
    /// Called when splash window wants to create/open/restore a project
    func onSplashAction(_ action: SplashAction) {
        Logger.shared.log("Splash action: \(action)")
        
        switch action {
        case .createProject(let project):
            currentProject = project
            settingsManager.activeProjectPath = project.directoryPath
            Logger.shared.log("Created project: \(project.name)")
            
        case .openProject(let project):
            currentProject = project
            settingsManager.activeProjectPath = project.directoryPath
            Logger.shared.log("Opened project: \(project.name)")
            
        case .restoreProject(let project):
            currentProject = project
            Logger.shared.log("Restored project: \(project.name)")
        }
        
        evaluateWindowState() // Re-evaluate after action
    }
    
    /// Called when settings window is closed
    func onSettingsClosed() {
        Logger.shared.log("Settings window closed")
        evaluateWindowState() // Re-evaluate permissions/agent state
    }
    
    /// Called when project window is closed
    func onProjectClosed() {
        Logger.shared.log("Project window closed")
        currentProject = nil
        settingsManager.activeProjectPath = ""
        evaluateWindowState() // Back to splash
    }
    
    /// Called when user selects New Project from menu
    private func onNewProjectMenuSelected() {
        currentProject = nil
        settingsManager.activeProjectPath = ""
        evaluateWindowState() // Should show splash for project creation
    }
    
    /// Called when user selects Open Project from menu  
    private func onOpenProjectMenuSelected() {
        currentProject = nil
        settingsManager.activeProjectPath = ""
        evaluateWindowState() // Should show splash for project selection
    }
    
    /// Called when user selects Preferences from menu
    private func onPreferencesMenuSelected() {
        // Force open settings regardless of current state
        dismissAllWindows()
        openWindow(id: "settings")
    }

    // === Debug and Utility Functions ===
    
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

// === Action Types for Window Communication ===

/// Actions that the splash window can trigger
enum SplashAction {
    case createProject(Project)
    case openProject(Project) 
    case restoreProject(Project)
}