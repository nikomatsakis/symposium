import SwiftUI

struct SplashView: View {
    @EnvironmentObject var startupManager: StartupManager
    
    // Callback to app for when startup completes
    var onStartupComplete: ((Bool, Bool) -> Void)?
    
    var body: some View {
        VStack(spacing: 32) {
            // Header
            VStack(spacing: 8) {
                Text("Symposium")
                    .font(.largeTitle)
                    .fontWeight(.bold)
                
                Text("Starting up...")
                    .font(.title3)
                    .foregroundColor(.secondary)
            }
            
            // Startup checks
            VStack(alignment: .leading, spacing: 16) {
                StartupCheckItemView(checkState: startupManager.permissionsCheck)
                StartupCheckItemView(checkState: startupManager.agentsCheck)
                StartupCheckItemView(checkState: startupManager.projectCheck)
            }
            .frame(maxWidth: 400, alignment: .leading)
            
            // Overall progress
            if !startupManager.startupComplete {
                ProgressView()
                    .scaleEffect(0.8)
            }
            
            Spacer()
        }
        .frame(
            minWidth: 500, idealWidth: 600, maxWidth: 700,
            minHeight: 400, idealHeight: 500, maxHeight: 600
        )
        .padding(32)
        .opacity(startupManager.splashVisible ? 1.0 : 0.0)
        .animation(.easeInOut(duration: 0.3), value: startupManager.splashVisible)
        .onAppear {
            Logger.shared.log("SplashView: onAppear - startup sequence beginning")
            startupManager.performStartupSequence()
        }
        .onDisappear {
            Logger.shared.log("SplashView: onDisappear")
        }
    }
}