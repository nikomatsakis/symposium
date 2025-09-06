import SwiftUI

/// Wrapper view for project windows that bridges between AppState and existing ProjectView
struct ProjectWindowView: View {
    let project: Project
    
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var agentManager: AgentManager
    @EnvironmentObject var settingsManager: SettingsManager
    @EnvironmentObject var permissionManager: PermissionManager
    
    var body: some View {
        // Create a ProjectManager for this project (temporary bridge)
        // TODO: Eventually we may want to move project management into AppState
        ProjectViewBridge(project: project)
            .environmentObject(appState)
            .environmentObject(agentManager)
            .environmentObject(settingsManager)
            .environmentObject(permissionManager)
    }
}

/// Bridge view that creates a ProjectManager and connects it to the existing ProjectView
private struct ProjectViewBridge: View {
    let project: Project
    
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var agentManager: AgentManager
    @EnvironmentObject var settingsManager: SettingsManager
    @EnvironmentObject var permissionManager: PermissionManager
    
    @StateObject private var projectManager: ProjectManager
    @StateObject private var ipcManager = IpcManager()
    
    init(project: Project) {
        self.project = project
        // Create ProjectManager for this project
        self._projectManager = StateObject(wrappedValue: ProjectManager(
            agentManager: AgentManager(), // Will be overridden by environmentObject
            settingsManager: SettingsManager(), // Will be overridden by environmentObject  
            selectedAgent: .claude, // TODO: Get from settings
            permissionManager: PermissionManager() // Will be overridden by environmentObject
        ))
    }
    
    var body: some View {
        ProjectView(projectManager: projectManager) {
            // onCloseProject callback - close this project in AppState
            Logger.shared.log("ProjectWindowView: Closing project via AppState")
            appState.closeProject(project)
        } onDismiss: {
            // onDismiss callback - for now, also close the project
            // Later we might want different behavior (e.g., just hide window)
            Logger.shared.log("ProjectWindowView: Dismissing project via AppState")
            appState.closeProject(project)
        }
        .onAppear {
            // Set up the project manager with the current project
            projectManager.currentProject = project
            Logger.shared.log("ProjectWindowView: Set current project in ProjectManager")
        }
    }
}