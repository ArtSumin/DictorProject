import SwiftUI

/// Custom toggle style matching AppColors.accentPrimary — avoids NSSwitch ignoring .tint() on macOS.
struct AccentToggleStyle: ToggleStyle {
    var onColor: Color = AppColors.accentPrimary
    var offColor: Color = Color.white.opacity(0.2)
    var thumbColor: Color = .white

    func makeBody(configuration: Configuration) -> some View {
        HStack {
            configuration.label
            Spacer()
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(configuration.isOn ? onColor : offColor)
                .frame(width: 50, height: 30)
                .overlay(
                    Circle()
                        .fill(thumbColor)
                        .frame(width: 22, height: 22)
                        .shadow(color: .black.opacity(0.2), radius: 1)
                        .padding(4)
                        .offset(x: configuration.isOn ? 20 : 0),
                    alignment: .leading
                )
                .onTapGesture {
                    withAnimation(.easeOut(duration: 0.2)) {
                        configuration.isOn.toggle()
                    }
                }
        }
    }
}
