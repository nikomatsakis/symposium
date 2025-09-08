import AppKit
import SwiftUI

enum AppWindowType {
    case splash
    case settings
    case createOrLoadProject
    case project
}

enum AppWindowState {
    case none
    case splash
    case settings
    case createOrLoadProject
    case project(Project)
}

@main
struct SymposiumApp: App {
    // SwiftUI environment for explicit window management
    @Environment(\.openWindow) private var openWindow
    @Environment(\.dismissWindow) private var dismissWindow

    // === State Machine Components ===
    @StateObject private var permissionManager = PermissionManager()
    @StateObject private var agentManager: AgentManager = AgentManager()
    @StateObject private var startupManager: StartupManager = StartupManager()

    // Path to the current project
    @AppStorage("activeProjectPath") var activeProjectPath: String? = nil

    // App delegate for dock click handling
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    init() {
        Logger.shared.log("App started")
    }

    var body: some Scene {
        // Startup window - pure startup progress display
        WindowGroup(id: "splash") {
            SplashView()
                .environmentObject(startupManager)
                .onAppear {
                    Logger.shared.log("Splash window appeared")
                    startupSequence()
                }
        }
        .windowResizability(.contentSize)
        .defaultAppStorage(.standard)

        // Settings window - permissions and agent selection
        WindowGroup(id: "settings") {
            SettingsView()
                .environmentObject(agentManager)
                .environmentObject(permissionManager)
                .onAppear {
                    Logger.shared.log("Settings window appeared")
                }
                .onDisappear {
                    // When the settings window is closed, we jump back to the splash screen.
                    Logger.shared.log("App: Settings window disappeared ~~> splash screen")
                    openWindow(id: "splash")
                }
        }
        .windowResizability(.contentSize)

        // Create/Load Project window - project selection and creation
        WindowGroup(id: "createOrLoadProject") {
            CreateOrLoadProjectView(
                onProjectCreated: handleProjectCreated,
                onProjectLoaded: handleProjectLoaded
            )
            .environmentObject(agentManager)
            .onAppear {
                Logger.shared.log("CreateOrLoadProject window appeared")
            }
        }
        .windowResizability(.contentSize)

        // Project window - active project with panel interface
        WindowGroup(id: "project") {
            if let project = currentProject {
                ProjectWindowView(project: project)
                    .environmentObject(agentManager)
                    .environmentObject(permissionManager)
                    .onAppear {
                        Logger.shared.log("Project window appeared: \(project.name)")
                    }
                    .onDisappear {
                        handleWindowClosed(.project)
                    }
            } 
        }
        .windowResizability(.contentSize)
        .windowLevel(.floating)  // Make project window float like the old NSPanel

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

    // === Startup sequence ===

    private func startupSequence() {
        Task {
            startupManager.reset()

            // First, check we have the required permissions
            if !permissionManager.checkAllPermissions() {
                await MainActor.run {
                    transitionToWindowState(.settings)
                }
                return
            }
            startupManager.permissionsCheck.completed = true

            // Check if we have a selected agent, scan if needed
            if !agentManager.checkSelectedAgent() {
                Logger.shared.log("App: No selected agent, scanning for available agents...")
                let agents = await agentManager.scanForAgents()
                Logger.shared.log("App: Agent scan completed, found \(agents.count) agents")

                await MainActor.run {
                    if agents.isEmpty || !agentManager.checkSelectedAgent() {
                        // Still no selected agent, go to settings
                        transitionToWindowState(.settings)
                        return
                    }
                }
            }
            startupManager.agentsCheck.completed = true

            // Check for saved project
            await MainActor.run {
                if let savedProject = loadSavedProject() {
                    transitionToWindowState(.project(savedProject))
                } else {
                    transitionToWindowState(.createOrLoadProject)
                }
                startupManager.projectCheck.completed = true
                startupManager.startupComplete = true
            }
        }
    }

    // === Centralized Window Management ===

    /// If we are not already in this window state, then transition, closing the current window
    func transitionToWindowState(_ newState: AppWindowState) {
        Logger.shared.log("App: Transitioning to window state: \(newState) from \(windowState)")
        if alreadyInWindowState(newState) {
            Logger.shared.log("App: Window already in state: \(newState)")
            return
        }

        dismissCurrentWindow()
        windowState = newState

        if let windowId = windowIdForState() {
            Logger.shared.log("App: opening window \(windowId)")
            openWindow(id: windowId)
        }
    }

    /// Handle window closures and decide what to do next
    func handleWindowClosed(_ closedWindow: AppWindowType) {
        Logger.shared.log("App: Window closed: \(closedWindow)")
        transitionToWindowState(.none)
        evaluateWindowState()
    }

    /// Load the saved project from settings if available and valid
    private func loadSavedProject() -> Project? {
        let savedPath = agentManager.activeProjectPath
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

    /// Evaluate what window state we should be in based on current conditions
    private func evaluateWindowState() {
        // Check permissions
        if !permissionManager.checkAllPermissions() {
            transitionToWindowState(.settings)
            return
        }

        // Check selected agent
        if !agentManager.checkSelectedAgent() {
            transitionToWindowState(.settings)
            return
        }

        // Check for current project
        if let project = currentProject {
            transitionToWindowState(.project(project))
        } else if let savedProject = loadSavedProject() {
            currentProject = savedProject
            transitionToWindowState(.project(savedProject))
        } else {
            transitionToWindowState(.createOrLoadProject)
        }
    }

    // === Explicit Window Management ===
    private func dismissAllWindows() {
        Logger.shared.log("Dismissing all windows for clean state")
        dismissWindow(id: "splash")
        dismissWindow(id: "settings")
        dismissWindow(id: "project")
    }

    // === Project Management ===

    /// Handle project creation from CreateOrLoadProjectView
    func handleProjectCreated(_ project: Project) {
        Logger.shared.log("App: Project created: \(project.name)")
        currentProject = project
        agentManager.activeProjectPath = project.directoryPath
        evaluateWindowState()
    }

    /// Handle project loading from CreateOrLoadProjectView
    func handleProjectLoaded(_ project: Project) {
        Logger.shared.log("App: Project loaded: \(project.name)")
        currentProject = project
        agentManager.activeProjectPath = project.directoryPath
        evaluateWindowState()
    }

    /// Called when user closes current project
    func closeCurrentProject() {
        Logger.shared.log("App: Closing current project")
        currentProject = nil
        agentManager.activeProjectPath = ""
        evaluateWindowState()
    }

    /// Called when user selects New Project from menu
    private func onNewProjectMenuSelected() {
        Logger.shared.log("Menu: New Project selected")
        closeCurrentProject()  // This will trigger transition to createOrLoadProject
    }

    /// Called when user selects Open Project from menu
    private func onOpenProjectMenuSelected() {
        Logger.shared.log("Menu: Open Project selected")
        closeCurrentProject()  // This will trigger transition to createOrLoadProject
    }

    /// Called when user selects Preferences from menu
    private func onPreferencesMenuSelected() {
        Logger.shared.log("Menu: Preferences selected")
        transitionToWindowState(.settings)
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
