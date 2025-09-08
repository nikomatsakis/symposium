import SwiftUI

struct StartupCheckItemView: View {
    @ObservedObject var check: StartupCheck

    init(check: StartupCheck) {
        self.check = check
    }

    var body: some View {
        HStack(spacing: 12) {
            // Icon
            Group {
                Image(systemName: check.completed ? "checkmark.square.fill" : "square")
                    .foregroundColor(check.completed ? .green : .secondary)
            }
            .frame(width: 16, height: 16)

            // Text
            Text(check.description)
                .font(.subheadline)
                .foregroundColor(check.completed ? .primary : .secondary)
        }
        .padding(.vertical, 4)
    }
}
