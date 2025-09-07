import SwiftUI

/// Wrapper view for project windows that creates a ProjectManager and connects to existing ProjectView
struct ProjectWindowView: View {
    let project: Project
    
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
            // onCloseProject callback - for now, just log
            // TODO: Connect to main app's onProjectClosed callback
            Logger.shared.log("ProjectWindowView: Project close requested")
        } onDismiss: {
            // onDismiss callback - for now, just log  
            // TODO: Connect to main app's onProjectClosed callback
            Logger.shared.log("ProjectWindowView: Project dismiss requested")
        }
        .onAppear {
            // Set up the project manager with the current project
            projectManager.currentProject = project
            Logger.shared.log("ProjectWindowView: Set current project in ProjectManager: \(project.name)")
        }
    }
}