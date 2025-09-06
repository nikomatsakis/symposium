import Foundation
import SwiftUI

/// Application-wide state that drives window visibility through reactive SwiftUI
@MainActor
class AppState: ObservableObject {
    /// List of projects that should have windows open
    /// When this array changes, SwiftUI will automatically show/hide windows
    @Published var openProjects: [Project] = []
    
    /// Whether the splash window should be visible (when no projects are open)
    var shouldShowSplash: Bool {
        openProjects.isEmpty
    }
    
    /// Open a project - adds it to openProjects array which triggers window creation
    func openProject(_ project: Project) {
        Logger.shared.log("AppState: Opening project: \(project.name)")
        
        // Avoid duplicates
        if !openProjects.contains(where: { $0.id == project.id }) {
            openProjects.append(project)
            Logger.shared.log("AppState: Project added to openProjects. Count: \(openProjects.count)")
        } else {
            Logger.shared.log("AppState: Project already open, not adding duplicate")
        }
    }
    
    /// Close a specific project - removes it from openProjects array
    func closeProject(_ project: Project) {
        Logger.shared.log("AppState: Closing project: \(project.name)")
        
        let beforeCount = openProjects.count
        openProjects.removeAll { $0.id == project.id }
        let afterCount = openProjects.count
        
        Logger.shared.log("AppState: Removed project. Before: \(beforeCount), After: \(afterCount)")
    }
    
    /// Close all projects - clears openProjects array, shows splash
    func closeAllProjects() {
        Logger.shared.log("AppState: Closing all projects. Current count: \(openProjects.count)")
        openProjects.removeAll()
        Logger.shared.log("AppState: All projects closed, splash should appear")
    }
    
    /// Get project by ID (useful for window identification)
    func project(withId id: UUID) -> Project? {
        return openProjects.first { $0.id == id }
    }
}