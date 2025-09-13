import Foundation

/// Window management modes for taskspace organization
enum WindowManagementMode: String, Codable, CaseIterable {
    case free = "free"
    case stack = "stack"
    case tile = "tile"
}

/// Version 0 project structure for backward compatibility
private struct ProjectV0: Codable {
    let id: UUID
    let name: String
    let gitURL: String
    let directoryPath: String
    var taskspaces: [Taskspace]
    let createdAt: Date
}

/// Version 2 project structure for migration from stackedWindowsEnabled to windowManagementMode
private struct ProjectV2: Codable {
    let version: Int
    let id: UUID
    let name: String
    let gitURL: String
    let directoryPath: String
    let agent: String?
    let defaultBranch: String?
    var taskspaces: [Taskspace]
    let createdAt: Date
    var stackedWindowsEnabled: Bool
}

/// Represents a Symposium project containing multiple taskspaces
struct Project: Codable, Identifiable {
    let version: Int
    let id: UUID
    let name: String
    let gitURL: String
    let directoryPath: String
    let agent: String?
    let defaultBranch: String?
    var taskspaces: [Taskspace] = []
    let createdAt: Date
    var windowManagementMode: WindowManagementMode = .free
    
    init(name: String, gitURL: String, directoryPath: String, agent: String? = nil, defaultBranch: String? = nil) {
        self.version = 3
        self.id = UUID()
        self.name = name
        self.gitURL = gitURL
        self.directoryPath = directoryPath
        self.agent = agent
        self.defaultBranch = defaultBranch
        self.createdAt = Date()
        self.windowManagementMode = .free
    }
    
    // Internal initializer for migration
    private init(version: Int, id: UUID, name: String, gitURL: String, directoryPath: String, agent: String?, defaultBranch: String?, taskspaces: [Taskspace], createdAt: Date, windowManagementMode: WindowManagementMode = .free) {
        self.version = version
        self.id = id
        self.name = name
        self.gitURL = gitURL
        self.directoryPath = directoryPath
        self.agent = agent
        self.defaultBranch = defaultBranch
        self.taskspaces = taskspaces
        self.createdAt = createdAt
        self.windowManagementMode = windowManagementMode
    }
    
    /// Path to project.json file
    var projectFilePath: String {
        return "\(directoryPath)/project.json"
    }
    
    /// Save project metadata to project.json
    func save() throws {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.outputFormatting = .prettyPrinted
        
        let data = try encoder.encode(self)
        try data.write(to: URL(fileURLWithPath: projectFilePath))
    }
    
    /// Load project from project.json file
    static func load(from directoryPath: String) throws -> Project {
        let projectFilePath = "\(directoryPath)/project.json"
        let data = try Data(contentsOf: URL(fileURLWithPath: projectFilePath))
        
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        
        do {
            // Try to decode with current schema (version 3)
            let project = try decoder.decode(Project.self, from: data)
            
            // No migration needed for version 3
            return project
        } catch {
            // Try version 2 migration
            do {
                let v2Project = try decoder.decode(ProjectV2.self, from: data)
                let windowMode: WindowManagementMode = v2Project.stackedWindowsEnabled ? .stack : .free
                
                let migratedProject = Project(
                    version: 3,
                    id: v2Project.id,
                    name: v2Project.name,
                    gitURL: v2Project.gitURL,
                    directoryPath: v2Project.directoryPath,
                    agent: v2Project.agent,
                    defaultBranch: v2Project.defaultBranch,
                    taskspaces: v2Project.taskspaces,
                    createdAt: v2Project.createdAt,
                    windowManagementMode: windowMode
                )
                
                // Save migrated project back to disk
                try migratedProject.save()
                return migratedProject
            } catch {
                // Fall back to legacy schema (version 0) and migrate
                let legacyProject = try decoder.decode(ProjectV0.self, from: data)
                let migratedProject = Project(
                    version: 3,
                    id: legacyProject.id,
                    name: legacyProject.name,
                    gitURL: legacyProject.gitURL,
                    directoryPath: legacyProject.directoryPath,
                    agent: nil,
                    defaultBranch: nil,
                    taskspaces: legacyProject.taskspaces,
                    createdAt: legacyProject.createdAt,
                    windowManagementMode: .free
                )
                
                // Save migrated project back to disk
                try migratedProject.save()
                
                return migratedProject
            }
        }
    }
    
    /// Check if directory contains a valid Symposium project
    static func isValidProjectDirectory(_ path: String) -> Bool {
        let projectFilePath = "\(path)/project.json"
        return FileManager.default.fileExists(atPath: projectFilePath)
    }
    
    /// Find taskspace by UUID
    func findTaskspace(uuid: String) -> Taskspace? {
        return taskspaces.first { $0.id.uuidString.lowercased() == uuid.lowercased() }
    }
    
    /// Find taskspace index by UUID
    func findTaskspaceIndex(uuid: String) -> Int? {
        return taskspaces.firstIndex { $0.id.uuidString.lowercased() == uuid.lowercased() }
    }
    
    /// Reorder taskspaces by most recently activated (most recent first)
    mutating func reorderTaskspacesByActivation() {
        taskspaces.sort { $0.lastActivatedAt > $1.lastActivatedAt }
    }
    
    /// Update taskspace activation time and reorder
    mutating func activateTaskspace(uuid: String) {
        guard let index = findTaskspaceIndex(uuid: uuid) else { return }
        taskspaces[index].updateActivationTime()
        reorderTaskspacesByActivation()
    }
}
