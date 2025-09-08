import SwiftUI

struct CreateOrLoadProjectView: View {
    @EnvironmentObject var agentManager: AgentManager

    let onProjectCreated: (Project) -> Void
    let onProjectLoaded: (Project) -> Void

    init(
        onProjectCreated: @escaping (Project) -> Void,
        onProjectLoaded: @escaping (Project) -> Void,
    ) {
        self.onProjectCreated = onProjectCreated
        self.onProjectLoaded = onProjectLoaded
    }

    var body: some View {
        VStack(spacing: 32) {
            // Header
            VStack(spacing: 8) {
                Text("Symposium")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("Choose how to get started")
                    .font(.title3)
                    .foregroundColor(.secondary)
            }

            // Action buttons
            VStack(spacing: 16) {
                // Create new project
                Button(action: createNewProject) {
                    HStack {
                        Image(systemName: "plus.circle.fill")
                            .font(.title2)

                        VStack(alignment: .leading, spacing: 4) {
                            Text("Create New Project")
                                .font(.headline)
                            Text("Start a fresh project with AI assistance")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        Spacer()
                    }
                    .padding()
                    .background(Color.accentColor.opacity(0.1))
                    .cornerRadius(12)
                }
                .buttonStyle(.plain)

                // Load existing project
                Button(action: loadExistingProject) {
                    HStack {
                        Image(systemName: "folder.circle.fill")
                            .font(.title2)

                        VStack(alignment: .leading, spacing: 4) {
                            Text("Open Existing Project")
                                .font(.headline)
                            Text("Continue working on a previous project")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        Spacer()
                    }
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(12)
                }
                .buttonStyle(.plain)
            }

            Spacer()
        }
        .padding(32)
        .frame(
            minWidth: 500, idealWidth: 600, maxWidth: 700,
            minHeight: 400, idealHeight: 500, maxHeight: 600
        )
    }

    private func createNewProject() {
        Logger.shared.log("CreateOrLoadProjectView: Creating new project")
        // TODO: Implement project creation flow
        // For now, create a dummy project
        let project = Project(
            name: "New Project",
            gitURL: "",
            directoryPath: "/tmp/new-project"
        )
        onProjectCreated(project)
    }

    private func loadExistingProject() {
        Logger.shared.log("CreateOrLoadProjectView: Loading existing project")
        // TODO: Implement file picker for .symposium project files
        // For now, create a dummy project
        let project = Project(
            name: "Loaded Project",
            gitURL: "",
            directoryPath: "/tmp/loaded-project"
        )
        onProjectLoaded(project)
    }
}

// Preview removed for compatibility
