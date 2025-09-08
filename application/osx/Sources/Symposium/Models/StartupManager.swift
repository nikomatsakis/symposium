import Combine
import Foundation
import SwiftUI

class StartupManager: ObservableObject {
    @Published var permissionsCheck = StartupCheck("Checking permissions")
    @Published var agentsCheck = StartupCheck("Checking AI agents")
    @Published var projectCheck = StartupCheck("Checking for previous project")
    @Published var startupComplete = false

    func reset() {
        permissionsCheck.reset()
        agentsCheck.reset()
        projectCheck.reset()
        startupComplete = false
    }
}

class StartupCheck: ObservableObject {
    let description: String
    @Published var inProgress: Bool = false
    @Published var completed: Bool = false

    init(_ description: String) {
        self.description = description
    }

    func reset() {
        inProgress = false
        completed = false
    }
}
