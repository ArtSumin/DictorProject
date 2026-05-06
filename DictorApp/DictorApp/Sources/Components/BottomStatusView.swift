import SwiftUI

struct BottomStatusView: View {
    let text: String
    let isConnected: Bool

    var body: some View {
        HStack(spacing: 6) {
            Text(text)
                .font(.system(size: 12, weight: .medium))
                .foregroundColor(.white.opacity(0.5))

            Circle()
                .fill(isConnected ? Color.green : Color.red)
                .frame(width: 7, height: 7)
        }
        .padding(.vertical, 12)
    }
}
