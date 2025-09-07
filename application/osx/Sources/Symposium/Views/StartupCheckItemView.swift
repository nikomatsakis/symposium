import SwiftUI

struct StartupCheckItemView: View {
    let checkState: StartupCheckState
    
    var body: some View {
        HStack(spacing: 12) {
            // Icon
            Group {
                switch checkState {
                case .pending:
                    Image(systemName: checkState.icon)
                        .foregroundColor(.secondary)
                case .inProgress:
                    ProgressView()
                        .scaleEffect(0.8)
                        .frame(width: 16, height: 16)
                case .completed:
                    Image(systemName: checkState.icon)
                        .foregroundColor(.green)
                case .failed:
                    Image(systemName: checkState.icon)
                        .foregroundColor(.red)
                }
            }
            .frame(width: 16, height: 16)
            
            // Text
            Text(checkState.displayText)
                .font(.subheadline)
                .foregroundColor(checkState.succeeded ? .primary : .secondary)
        }
        .padding(.vertical, 4)
    }
}

// Preview removed for compatibility