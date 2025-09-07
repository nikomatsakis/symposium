import SwiftUI

struct SplashView: View {
    // Reference to the main app for callbacks
    let app: SymposiumApp
    
    @EnvironmentObject var permissionManager: PermissionManager
    @EnvironmentObject var settingsManager: SettingsManager
    @EnvironmentObject var agentManager: AgentManager
    @EnvironmentObject var appDelegate: AppDelegate
    @State private var showingSettings = false

    var body: some View {
        VStack {
            // Simple header bar - Settings button only
            HStack {
                Spacer()

                Button("Settings") {
                    showingSettings = true
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)

            // Main content
            VStack {
                if !permissionManager.hasAccessibilityPermission
                    || !permissionManager.hasScreenRecordingPermission
                {
                    // Show settings if required permissions are missing
                    SettingsView()
                        .onAppear {
                            Logger.shared.log(
                                "SplashView: Showing SettingsView - missing permissions")
                        }
                } else if !agentManager.scanningCompleted && agentManager.scanningInProgress {
                    // Show loading while scanning agents
                    VStack(spacing: 16) {
                        ProgressView()
                            .scaleEffect(1.2)

                        Text("Scanning for agents...")
                            .font(.headline)
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .onAppear {
                        Logger.shared.log(
                            "SplashView: Showing agent scanning - scanningInProgress: \(agentManager.scanningInProgress)"
                        )
                    }
                } else {
                    // Show project selection when no active project
                    ProjectSelectionView(
                        onProjectCreated: { projectManager in
                            Logger.shared.log("SplashView: Project created via callback")
                            handleProjectCreated(projectManager)
                        }
                    )
                    .onAppear {
                        Logger.shared.log(
                            "SplashView: Showing ProjectSelectionView - permissions OK, scanning done"
                        )
                    }
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .sheet(isPresented: $showingSettings) {
            SettingsView()
        }
        .onChange(of: agentManager.scanningCompleted) { completed in
            if completed {
                Logger.shared.log("SplashView: Agent scanning completed, checking for last project")
                checkForLastProject()
            }
        }
        .onAppear {
            Logger.shared.log("SplashView: onAppear - triggering evaluateWindowState")
            app.evaluateWindowState()
        }
    }
    
    // === Project Management ===
    
    private func handleProjectCreated(_ projectManager: ProjectManager) {
        Logger.shared.log("SplashView: Handling project creation")
        
        guard let project = projectManager.currentProject else {
            Logger.shared.log("SplashView: ERROR - ProjectManager has no current project")
            return
        }
        
        // Notify the main app via callback
        Logger.shared.log("SplashView: Notifying app of created project: \(project.name)")
        app.onSplashAction(.createProject(project))
    }
    
    private func checkForLastProject() {
        Logger.shared.log(
            "SplashView: checkForLastProject - activeProjectPath: '\(settingsManager.activeProjectPath)'"
        )
        Logger.shared.log(
            "SplashView: checkForLastProject - hasAccessibility: \(permissionManager.hasAccessibilityPermission)"
        )
        Logger.shared.log(
            "SplashView: checkForLastProject - hasScreenRecording: \(permissionManager.hasScreenRecordingPermission)"
        )
        Logger.shared.log(
            "SplashView: checkForLastProject - agentsAvailable: \(agentManager.scanningCompleted)")

        // If we have a valid active project and permissions are OK, restore it
        if !settingsManager.activeProjectPath.isEmpty,
            permissionManager.hasAccessibilityPermission,
            permissionManager.hasScreenRecordingPermission,
            agentManager.scanningCompleted
        {

            Logger.shared.log(
                "SplashView: Found active project, restoring: \(settingsManager.activeProjectPath)"
            )
            
            // Attempt to restore the active project
            let restoredProjectManager = ProjectManager(
                agentManager: agentManager,
                settingsManager: settingsManager, 
                selectedAgent: settingsManager.selectedAgent,
                permissionManager: permissionManager
            )
            
            do {
                try restoredProjectManager.openProject(at: settingsManager.activeProjectPath)
                
                guard let project = restoredProjectManager.currentProject else {
                    Logger.shared.log("SplashView: ERROR - Restored project manager has no current project")
                    return
                }
                
                Logger.shared.log("SplashView: Successfully restored project, notifying app")
                app.onSplashAction(.restoreProject(project))
            } catch {
                Logger.shared.log("SplashView: Failed to restore active project: \(error)")
                // Clear invalid active project path
                settingsManager.activeProjectPath = ""
            }
        } else {
            Logger.shared.log("SplashView: No active project to restore - showing project selection")
        }
    }
}