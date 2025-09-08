import AppKit
import Foundation
import SwiftUI

enum AgentType: String, CaseIterable, Identifiable, Codable {
    case qcli = "qcli"
    case claude = "claude"

    var id: String { rawValue }

    var displayName: String {
        switch self {
        case .qcli: return "Amazon Q CLI"
        case .claude: return "Claude Code"
        }
    }
}

class AgentManager: ObservableObject {
    @AppStorage("selectedAgent") var selectedAgentRaw: String = ""

    // Note: We store `cachedAgents` as Data because @AppStorage only supports primitive types directly.
    // Each access re-decodes from JSON (no memoization), but this is acceptable since we
    // only read once at startup and write once after each agent scan.
    @AppStorage("cachedAgents") private var cachedAgentsData: Data = Data()

    // Date of last scan
    //
    // Stored as time internal since 1970 so that it can be optional
    @AppStorage("lastScanDate") var lastScanInterval: Double = 0

    @Published var scanningInProgress = false

    init() {
        Logger.shared.log("AgentManager: Created")
    }

    var selectedAgent: AgentType? {
        get { if selectedAgentRaw.isEmpty { nil } else { AgentType(rawValue: selectedAgentRaw) } }
        set { selectedAgentRaw = newValue?.rawValue ?? "" }
    }

    var cachedAgents: [AgentInfo] {
        get {
            guard !cachedAgentsData.isEmpty else { return [] }
            do {
                return try JSONDecoder().decode([AgentInfo].self, from: cachedAgentsData)
            } catch {
                Logger.shared.log("SettingsManager: Failed to decode cached agents: \(error)")
                return []
            }
        }
        set {
            do {
                cachedAgentsData = try JSONEncoder().encode(newValue)
                lastScanDate = Date()
                Logger.shared.log("SettingsManager: Cached \(newValue.count) agents")
            } catch {
                Logger.shared.log("SettingsManager: Failed to encode agents for caching: \(error)")
            }
        }
    }

    var lastScanDate: Date? {
        get { lastScanInterval == 0 ? nil : Date(timeIntervalSince1970: lastScanInterval) }
        set { lastScanInterval = newValue?.timeIntervalSince1970 ?? 0 }
    }

    /// Check if we have a selected agent, returns true if we do
    func checkSelectedAgent() -> Bool {
        Logger.shared.log("AgentManager: checkSelectedAgent")
        if let a = selectedAgent {
            Logger.shared.log("AgentManager: found selected agent \(a)")
            return true
        }
        return false
    }

    /// Async agent scan that you can await
    @MainActor
    func scanForAgents() async -> [AgentInfo] {
        if scanningInProgress {
            Logger.shared.log("AgentManager: Agent scan already in progress")
            // Wait for the current scan to complete
            while scanningInProgress {
                try? await Task.sleep(nanoseconds: 100_000_000)  // 0.1 seconds
            }
            return cachedAgents
        }

        Logger.shared.log("AgentManager: Starting agent scan")
        scanningInProgress = true

        let agents = await withTaskGroup(of: AgentInfo?.self) { group in
            var results: [AgentInfo] = []

            // Check for Q CLI
            group.addTask {
                Logger.shared.log("AgentManager: Checking for Q CLI...")
                let qcliInfo = await self.detectQCLI()
                if let qcliInfo = qcliInfo {
                    Logger.shared.log("AgentManager: Q CLI detected: \(qcliInfo.statusText)")
                } else {
                    Logger.shared.log("AgentManager: Q CLI not found")
                }
                return qcliInfo
            }

            // Check for Claude Code
            group.addTask {
                Logger.shared.log("AgentManager: Checking for Claude Code...")
                let claudeInfo = await self.detectClaudeCode()
                if let claudeInfo = claudeInfo {
                    Logger.shared.log(
                        "AgentManager: Claude Code detected: \(claudeInfo.statusText)")
                } else {
                    Logger.shared.log("AgentManager: Claude Code not found")
                }
                return claudeInfo
            }

            for await agent in group {
                if let agent = agent {
                    results.append(agent)
                }
            }

            return results
        }

        cachedAgents = agents
        scanningInProgress = false

        Logger.shared.log("AgentManager: Scan complete. Found \(agents.count) agents.")
        return agents
    }

    private func detectQCLI() async -> AgentInfo? {
        // Check if q command exists in PATH
        let qPath = await findExecutable(name: "q")
        guard let path = qPath else { return nil }

        // Verify it's actually Q CLI by checking version
        let version = await getQCLIVersion(path: path)

        // Check if MCP is configured and get the path
        let (mcpConfigured, mcpPath) = await checkQCLIMCPConfiguration(qPath: path)

        return AgentInfo(
            type: .qcli,
            name: "Q CLI",
            description: "Amazon Q Developer CLI",
            executablePath: path,
            version: version,
            isInstalled: true,
            isMCPConfigured: mcpConfigured,
            mcpServerPath: mcpPath
        )
    }

    private func detectClaudeCode() async -> AgentInfo? {
        Logger.shared.log("AgentManager: Looking for Claude Code executable...")

        // First try to find claude in PATH
        if let path = await findExecutable(name: "claude") {
            Logger.shared.log("AgentManager: Found claude at: \(path)")
            let version = await getClaudeCodeVersion(path: path)
            Logger.shared.log("AgentManager: Claude version: \(version ?? "unknown")")
            let (mcpConfigured, mcpPath) = await checkClaudeCodeMCPConfiguration(claudePath: path)

            return AgentInfo(
                type: .claude,
                name: "Claude Code",
                description: "Anthropic Claude for coding",
                executablePath: path,
                version: version,
                isInstalled: true,
                isMCPConfigured: mcpConfigured,
                mcpServerPath: mcpPath
            )
        }

        Logger.shared.log("AgentManager: Claude not found in PATH, checking common locations...")

        // If not found in PATH, check common installation paths
        let possiblePaths = [
            "/usr/local/bin/claude",
            "/opt/homebrew/bin/claude",
            "~/.local/bin/claude",
            "~/.volta/bin/claude",
        ].map { NSString(string: $0).expandingTildeInPath }

        for path in possiblePaths {
            Logger.shared.log("AgentManager: Checking: \(path)")
            if FileManager.default.isExecutableFile(atPath: path) {
                Logger.shared.log("AgentManager: Found executable at: \(path)")
                let version = await getClaudeCodeVersion(path: path)
                let (mcpConfigured, mcpPath) = await checkClaudeCodeMCPConfiguration(
                    claudePath: path)

                return AgentInfo(
                    type: .claude,
                    name: "Claude Code",
                    description: "Anthropic Claude for coding",
                    executablePath: path,
                    version: version,
                    isInstalled: true,
                    isMCPConfigured: mcpConfigured,
                    mcpServerPath: mcpPath
                )
            }
        }

        Logger.shared.log("AgentManager: Claude Code not found anywhere")

        // Return not installed info
        return AgentInfo(
            type: .claude,
            name: "Claude Code",
            description: "Anthropic Claude for coding",
            executablePath: nil,
            version: nil,
            isInstalled: false,
            isMCPConfigured: false,
            mcpServerPath: nil
        )
    }

    private func findExecutable(name: String) async -> String? {
        return await withCheckedContinuation { continuation in
            let process = Process()
            process.launchPath = "/usr/bin/which"
            process.arguments = [name]

            let pipe = Pipe()
            process.standardOutput = pipe
            process.standardError = Pipe()

            process.terminationHandler = { process in
                if process.terminationStatus == 0 {
                    let data = pipe.fileHandleForReading.readDataToEndOfFile()
                    let output = String(data: data, encoding: .utf8)?.trimmingCharacters(
                        in: .whitespacesAndNewlines)
                    continuation.resume(returning: output?.isEmpty == false ? output : nil)
                } else {
                    continuation.resume(returning: nil)
                }
            }

            do {
                try process.run()
            } catch {
                print("Error finding executable \(name): \(error)")
                continuation.resume(returning: nil)
            }
        }
    }

    private func getQCLIVersion(path: String) async -> String? {
        return await runCommand(path: path, arguments: ["--version"])
    }

    private func getClaudeCodeVersion(path: String) async -> String? {
        return await runCommand(path: path, arguments: ["--version"])
    }

    private func runCommand(path: String, arguments: [String]) async -> String? {
        return await withCheckedContinuation { continuation in
            let process = Process()
            process.launchPath = path
            process.arguments = arguments

            let stdoutPipe = Pipe()
            let stderrPipe = Pipe()
            process.standardOutput = stdoutPipe
            process.standardError = stderrPipe

            process.terminationHandler = { process in
                // Try stdout first
                let stdoutData = stdoutPipe.fileHandleForReading.readDataToEndOfFile()
                let stdoutOutput = String(data: stdoutData, encoding: .utf8)?.trimmingCharacters(
                    in: .whitespacesAndNewlines)

                if let stdout = stdoutOutput, !stdout.isEmpty {
                    continuation.resume(returning: stdout)
                    return
                }

                // If stdout is empty, try stderr (Q CLI outputs to stderr)
                let stderrData = stderrPipe.fileHandleForReading.readDataToEndOfFile()
                let stderrOutput = String(data: stderrData, encoding: .utf8)?.trimmingCharacters(
                    in: .whitespacesAndNewlines)

                continuation.resume(returning: stderrOutput?.isEmpty == false ? stderrOutput : nil)
            }

            do {
                try process.run()
            } catch {
                print("Error running command \(path): \(error)")
                continuation.resume(returning: nil)
            }
        }
    }

    private func checkQCLIMCPConfiguration(qPath: String) async -> (Bool, String?) {
        // Use Q CLI's built-in MCP status command to check for symposium-mcp
        let output = await runCommand(
            path: qPath, arguments: ["mcp", "status", "--name", "symposium"])

        guard let output = output, !output.isEmpty else {
            return (false, nil)
        }

        // Parse the output to extract the Command path
        // Look for lines like "Command : /path/to/symposium-mcp"
        let lines = output.components(separatedBy: .newlines)
        for line in lines {
            if line.contains("Command :") {
                let parts = line.components(separatedBy: ":")
                if parts.count >= 2 {
                    let path = parts[1].trimmingCharacters(in: .whitespaces)
                    return (true, path)
                }
            }
        }

        // Found output but couldn't parse path
        return (true, nil)
    }

    private func checkClaudeCodeMCPConfiguration(claudePath: String) async -> (Bool, String?) {
        // Use Claude Code's built-in MCP list command to check for symposium-mcp
        let output = await runCommand(path: claudePath, arguments: ["mcp", "list"])

        Logger.shared.log("AgentManager: Claude MCP command: \(claudePath) mcp list")
        Logger.shared.log("AgentManager: Claude MCP output: \(output ?? "nil")")

        guard let output = output, !output.isEmpty else {
            return (false, nil)
        }

        // Parse the output to find symposium entry
        // Look for lines like "symposium: /path/to/symposium-mcp --dev-log - ✓ Connected"
        let lines = output.components(separatedBy: .newlines)
        for line in lines {
            if line.contains("symposium:") && line.contains("✓ Connected") {
                // Extract the path between "symposium: " and " --dev-log"
                let parts = line.components(separatedBy: ":")
                if parts.count >= 2 {
                    let pathPart = parts[1].trimmingCharacters(in: .whitespaces)
                    // Split by " --" to get just the path
                    let pathComponents = pathPart.components(separatedBy: " --")
                    if let path = pathComponents.first?.trimmingCharacters(in: .whitespaces) {
                        return (true, path)
                    }
                }
            }
        }

        // Check if symposium is listed but not connected
        for line in lines {
            if line.contains("symposium:") {
                return (false, nil)  // Found but not connected
            }
        }

        return (false, nil)
    }

    private func readMCPConfig(path: String) -> String? {
        guard FileManager.default.fileExists(atPath: path) else { return nil }

        do {
            return try String(contentsOfFile: path, encoding: .utf8)
        } catch {
            print("Error reading MCP config at \(path): \(error)")
            return nil
        }
    }
}

struct AgentInfo: Identifiable, Codable {
    let type: AgentType
    let name: String
    let description: String
    let executablePath: String?
    let version: String?
    let isInstalled: Bool
    let isMCPConfigured: Bool
    let mcpServerPath: String?

    var id: String { type.id }

    var statusText: String {
        if !isInstalled {
            return "Not installed"
        } else if !isMCPConfigured {
            return "MCP not configured"
        } else {
            return "Ready"
        }
    }

    var statusColor: NSColor {
        if !isInstalled {
            return .systemRed
        } else if !isMCPConfigured {
            return .systemOrange
        } else {
            return .systemGreen
        }
    }

    var statusIcon: String {
        if !isInstalled {
            return "xmark.circle.fill"
        } else if !isMCPConfigured {
            return "exclamationmark.triangle.fill"
        } else {
            return "checkmark.circle.fill"
        }
    }

    /// Generate command for hatchling taskspace with initial prompt
    func getHatchlingCommand(initialPrompt: String) -> [String]? {
        guard isInstalled && isMCPConfigured else { return nil }

        switch type {
        case .qcli:
            return ["q", "chat", initialPrompt]
        case .claude:
            // TODO: Implement claude-code hatchling command
            return nil
        }
    }

    /// Generate command for resume taskspace
    func getResumeCommand() -> [String]? {
        guard isInstalled && isMCPConfigured else { return nil }

        switch id {
        case "qcli":
            return ["q", "chat", "--resume"]
        case "claude-code":
            // TODO: Implement claude-code resume command
            return nil
        default:
            return nil
        }
    }
}

extension AgentManager {

    /// Get agent command for a taskspace based on its state and selected agent
    func getAgentCommand(for taskspace: Taskspace, selectedAgent: AgentType) -> [String]? {
        guard let agentInfo = cachedAgents.first(where: { $0.type == selectedAgent }) else {
            return nil
        }

        switch taskspace.state {
        case .hatchling(let initialPrompt):
            return agentInfo.getHatchlingCommand(initialPrompt: initialPrompt)

        case .resume:
            return agentInfo.getResumeCommand()
        }
    }
}
