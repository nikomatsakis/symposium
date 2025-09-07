import Foundation
import SwiftUI

class SettingsManager: ObservableObject {
    @AppStorage("selectedAgent") var selectedAgentRaw: String = AgentType.qcli.rawValue
    @AppStorage("activeProjectPath") var activeProjectPath: String = ""
    @AppStorage("cachedAgents") private var cachedAgentsData: Data = Data()
    @AppStorage("lastAgentScanTime") var lastAgentScanTime: Double = 0
    
    var selectedAgent: AgentType {
        get { AgentType(rawValue: selectedAgentRaw) ?? .qcli }
        set { selectedAgentRaw = newValue.rawValue }
    }
    
    var lastAgentScanDate: Date? {
        get { lastAgentScanTime > 0 ? Date(timeIntervalSince1970: lastAgentScanTime) : nil }
        set { lastAgentScanTime = newValue?.timeIntervalSince1970 ?? 0 }
    }
    
    // Cached agents persistence
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
                lastAgentScanDate = Date()
                Logger.shared.log("SettingsManager: Cached \(newValue.count) agents")
            } catch {
                Logger.shared.log("SettingsManager: Failed to encode agents for caching: \(error)")
            }
        }
    }
}
