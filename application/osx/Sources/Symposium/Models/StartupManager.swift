import Foundation
import Combine
import SwiftUI

enum StartupCheckState {
    case pending(String)      // "Checking permissions..."
    case inProgress(String)   // "Scanning agents..."
    case completed(String)    // "Found 2 agents"
    case failed(String)       // "Missing permissions"
    
    var isComplete: Bool {
        switch self {
        case .completed, .failed: return true
        case .pending, .inProgress: return false
        }
    }
    
    var succeeded: Bool {
        switch self {
        case .completed: return true
        default: return false
        }
    }
    
    var displayText: String {
        switch self {
        case .pending(let text), .inProgress(let text), 
             .completed(let text), .failed(let text):
            return text
        }
    }
    
    var icon: String {
        switch self {
        case .pending: return "clock"
        case .inProgress: return "arrow.clockwise"
        case .completed: return "checkmark.circle.fill"
        case .failed: return "xmark.circle.fill"
        }
    }
}

class StartupManager: ObservableObject {
    @ObservedObject var permissionManager: PermissionManager
    @ObservedObject var agentManager: AgentManager
    @ObservedObject var settingsManager: SettingsManager
    
    @Published var splashVisible = true
    private var cancellables = Set<AnyCancellable>()
    
    init(permissionManager: PermissionManager, agentManager: AgentManager, settingsManager: SettingsManager) {
        self.permissionManager = permissionManager
        self.agentManager = agentManager
        self.settingsManager = settingsManager
        
        // Set up dependency tracking - when any manager changes, we update
        permissionManager.objectWillChange.sink { [weak self] in
            self?.objectWillChange.send()
        }.store(in: &cancellables)
        
        agentManager.objectWillChange.sink { [weak self] in
            self?.objectWillChange.send()
        }.store(in: &cancellables)
        
        settingsManager.objectWillChange.sink { [weak self] in
            self?.objectWillChange.send()
        }.store(in: &cancellables)
    }
    
    // Computed properties that SwiftUI will reactively update
    var permissionsCheck: StartupCheckState {
        if !permissionManager.hasAccessibilityPermission || !permissionManager.hasScreenRecordingPermission {
            return .failed("Missing permissions - click to configure")
        }
        return .completed("Permissions granted")
    }
    
    var agentsCheck: StartupCheckState {
        if agentManager.scanningInProgress {
            return .inProgress("Scanning for agents...")
        } else if !agentManager.scanningCompleted {
            return .pending("Waiting to scan agents...")
        } else if agentManager.availableAgents.isEmpty {
            return .failed("No agents found - click to configure")
        } else {
            let validAgents = agentManager.availableAgents.filter { $0.isInstalled && $0.isMCPConfigured }
            if validAgents.isEmpty {
                return .failed("Agents need configuration")
            }
            return .completed("Found \(validAgents.count) configured agents")
        }
    }
    
    var projectCheck: StartupCheckState {
        let savedPath = settingsManager.activeProjectPath
        if savedPath.isEmpty {
            return .completed("No previous project")
        } else {
            // TODO: Actually validate the project exists
            return .completed("Previous project available")
        }
    }
    
    var allChecks: [StartupCheckState] {
        [permissionsCheck, agentsCheck, projectCheck]
    }
    
    var startupComplete: Bool {
        allChecks.allSatisfy(\.isComplete)
    }
    
    var shouldShowSettings: Bool {
        !permissionsCheck.succeeded || !agentsCheck.succeeded
    }
    
    var hasActiveProject: Bool {
        projectCheck.displayText.contains("Previous project available")
    }
    
    // Callback for when startup completes
    var onStartupComplete: ((Bool, Bool) -> Void)? = nil
    
    // Trigger startup sequence
    func performStartupSequence() {
        Logger.shared.log("StartupManager: Beginning startup sequence")
        
        // The startup checks are computed properties that will update automatically
        // as the underlying managers complete their work. We just need to ensure
        // the managers start their work.
        
        // PermissionManager checks permissions in its init() already
        
        // AgentManager either loads cached agents or starts scanning in its init() already
        
        // Monitor for completion
        monitorForCompletion()
        
        Logger.shared.log("StartupManager: Startup sequence initiated - waiting for managers to complete")
    }
    
    private func monitorForCompletion() {
        // Use a timer or observe changes to detect completion
        Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { timer in
            if self.startupComplete {
                timer.invalidate()
                Logger.shared.log("StartupManager: Startup sequence completed")
                
                // Hide splash and notify callback
                DispatchQueue.main.async {
                    self.splashVisible = false
                    self.onStartupComplete?(self.shouldShowSettings, self.hasActiveProject)
                }
            }
        }
    }
}