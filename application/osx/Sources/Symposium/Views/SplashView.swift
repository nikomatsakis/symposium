import SwiftUI

struct SplashView: View {
    @EnvironmentObject var startupManager: StartupManager

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
                StartupCheckItemView(check: startupManager.permissionsCheck)
                StartupCheckItemView(check: startupManager.agentsCheck)
                StartupCheckItemView(check: startupManager.projectCheck)
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
    }
}
