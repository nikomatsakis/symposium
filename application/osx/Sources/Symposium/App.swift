import AppKit
import SwiftUI

enum AppWindowType {
    case splash
    case settings  
    case createOrLoadProject
    case project
}

enum AppWindowState {
    case splash
    case settings  
    case createOrLoadProject
    case project(Project)
}

@main
struct SymposiumApp: App {
    // === State Machine Components ===
    @StateObject private var settingsManager = SettingsManager()
    @StateObject private var permissionManager = PermissionManager()
    @StateObject private var agentManager: AgentManager
    @StateObject private var startupManager: StartupManager
    
    init() {
        // Create managers with proper dependencies
        let settings = SettingsManager()
        let permissions = PermissionManager()
        let agents = AgentManager(settingsManager: settings)
        let startup = StartupManager(
            permissionManager: permissions, 
            agentManager: agents, 
            settingsManager: settings
        )
        
        self._settingsManager = StateObject(wrappedValue: settings)
        self._permissionManager = StateObject(wrappedValue: permissions)
        self._agentManager = StateObject(wrappedValue: agents)
        self._startupManager = StateObject(wrappedValue: startup)
    }
    
    // Window state management
    @State private var windowState: AppWindowState = .splash
    @State private var currentProject: Project? = nil
    
    // App delegate for dock click handling
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    // SwiftUI environment for explicit window management
    @Environment(\.openWindow) private var openWindow
    @Environment(\.dismissWindow) private var dismissWindow
    
    var body: some Scene {
        // Splash window - pure startup progress display
        WindowGroup(id: "splash") {
            SplashView()
                .environmentObject(startupManager)
                .onAppear {
                    Logger.shared.log("Splash window appeared")
                }
                .onDisappear {
                    handleWindowClosed(.splash)
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
                .onDisappear {
                    handleWindowClosed(.settings)
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
            .environmentObject(settingsManager)
            .onAppear {
                Logger.shared.log("CreateOrLoadProject window appeared")
            }
            .onDisappear {
                handleWindowClosed(.createOrLoadProject)
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
                        handleWindowClosed(.project)
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
    
    // === Centralized Window Management ===
    
    /// Centralized window transition with state cleanup
    func transitionTo(_ newState: AppWindowState) {
        Logger.shared.log("App: Transitioning to window state: \(newState)")
        windowState = newState
        
        // Close all windows first
        dismissWindow(id: "splash")
        dismissWindow(id: "settings") 
        dismissWindow(id: "createOrLoadProject")
        dismissWindow(id: "project")
        
        // Open the appropriate window
        switch newState {
        case .splash: 
            openWindow(id: "splash")
        case .settings: 
            openWindow(id: "settings")
        case .createOrLoadProject: 
            openWindow(id: "createOrLoadProject")
        case .project(let project):
            currentProject = project
            openWindow(id: "project")
        }
    }
    
    /// Handle window closures and decide what to do next
    func handleWindowClosed(_ closedWindow: AppWindowType) {
        Logger.shared.log("App: Window closed: \(closedWindow)")
        
        // Update startup manager based on current state and decide next action
        Task { @MainActor in
            // Wait a brief moment for state to settle
            try? await Task.sleep(nanoseconds: 100_000_000) // 0.1 seconds
            
            if startupManager.startupComplete {
                // Startup is done, make decisions based on configuration state
                if startupManager.shouldShowSettings {
                    transitionTo(.settings)
                } else if startupManager.hasActiveProject {
                    // TODO: Load the actual project
                    let dummyProject = Project(name: "Restored Project", gitURL: "", directoryPath: "/tmp")
                    transitionTo(.project(dummyProject))
                } else {
                    transitionTo(.createOrLoadProject)
                }
            } else {
                // Startup not complete, show splash
                startupManager.splashVisible = true
                transitionTo(.splash)
            }
        }
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
    
    // === Project Management ===
    
    /// Handle project creation from CreateOrLoadProjectView
    func handleProjectCreated(_ project: Project) {
        Logger.shared.log("App: Project created: \(project.name)")
        currentProject = project
        settingsManager.activeProjectPath = project.directoryPath
        
        // Hide splash and transition to project
        startupManager.splashVisible = false
        transitionTo(.project(project))
    }
    
    /// Handle project loading from CreateOrLoadProjectView
    func handleProjectLoaded(_ project: Project) {
        Logger.shared.log("App: Project loaded: \(project.name)")
        currentProject = project
        settingsManager.activeProjectPath = project.directoryPath
        
        // Hide splash and transition to project
        startupManager.splashVisible = false
        transitionTo(.project(project))
    }
    
    /// Called when user closes current project
    func closeCurrentProject() {
        Logger.shared.log("App: Closing current project")
        currentProject = nil
        settingsManager.activeProjectPath = ""
        
        // This will trigger onDisappear -> handleWindowClosed -> transition logic
        dismissWindow(id: "project")
    }
    
    /// Called when user selects New Project from menu
    private func onNewProjectMenuSelected() {
        Logger.shared.log("Menu: New Project selected")
        closeCurrentProject() // This will trigger transition to createOrLoadProject
    }
    
    /// Called when user selects Open Project from menu  
    private func onOpenProjectMenuSelected() {
        Logger.shared.log("Menu: Open Project selected")
        closeCurrentProject() // This will trigger transition to createOrLoadProject
    }
    
    /// Called when user selects Preferences from menu
    private func onPreferencesMenuSelected() {
        Logger.shared.log("Menu: Preferences selected")
        transitionTo(.settings)
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

