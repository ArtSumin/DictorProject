import SwiftUI

struct TopBarView: View {
    var onHistoryTap: () -> Void = {}
    var onSettingsTap: () -> Void = {}
    var onClose: () -> Void = {}

    @State private var isHistoryHovered = false
    @State private var isSettingsHovered = false
    @State private var isCloseHovered = false

    var body: some View {
        HStack(spacing: 12) {
            Button(action: onHistoryTap) {
                HStack(spacing: 6) {
                    Image(systemName: "clock.arrow.circlepath")
                        .font(.system(size: 12))
                    Text("History")
                        .font(.system(size: 13, weight: .medium))
                }
                .foregroundColor(.white.opacity(isHistoryHovered ? 0.95 : 0.7))
                .animation(.easeOut(duration: 0.12), value: isHistoryHovered)
            }
            .buttonStyle(.plain)
            .onHover { isHistoryHovered = $0 }

            Spacer()

            Button(action: onSettingsTap) {
                Image(systemName: "gearshape")
                    .font(.system(size: 14))
                    .foregroundColor(.white.opacity(isSettingsHovered ? 0.95 : 0.7))
                    .rotationEffect(.degrees(isSettingsHovered ? 30 : 0))
                    .animation(.easeOut(duration: 0.2), value: isSettingsHovered)
            }
            .buttonStyle(.plain)
            .onHover { isSettingsHovered = $0 }

            Button(action: onClose) {
                Image(systemName: "xmark")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundColor(.white.opacity(isCloseHovered ? 0.95 : 0.5))
                    .frame(width: 20, height: 20)
                    .background(
                        Circle()
                            .fill(Color.white.opacity(isCloseHovered ? 0.12 : 0.06))
                    )
                    .animation(.easeOut(duration: 0.12), value: isCloseHovered)
            }
            .buttonStyle(.plain)
            .onHover { isCloseHovered = $0 }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }
}
