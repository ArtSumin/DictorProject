import SwiftUI

struct IdleView: View {
    var onStartRecording: () -> Void

    @State private var isPulsing = false
    @State private var isButtonHovered = false

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            microphoneArea

            Text("Tap to speak")
                .font(.system(size: 14, weight: .medium))
                .foregroundColor(.white.opacity(0.45))
                .padding(.top, 16)

            Spacer()

            startButton
                .padding(.bottom, 8)
        }
        .onAppear { isPulsing = true }
    }

    // MARK: - Microphone with concentric circles

    private var microphoneArea: some View {
        ZStack {
            ForEach(0..<3, id: \.self) { ring in
                Circle()
                    .stroke(
                        AppColors.accentPrimary.opacity(0.18 - Double(ring) * 0.05),
                        lineWidth: 1.2
                    )
                    .frame(
                        width: 110 + CGFloat(ring) * 40,
                        height: 110 + CGFloat(ring) * 40
                    )
                    .scaleEffect(isPulsing ? 1.0 + CGFloat(ring + 1) * 0.015 : 1.0)
                    .opacity(isPulsing ? 0.6 : 1.0)
                    .animation(
                        .easeInOut(duration: 1.8 + Double(ring) * 0.3)
                            .repeatForever(autoreverses: true),
                        value: isPulsing
                    )
            }

            Circle()
                .fill(
                    RadialGradient(
                        colors: [
                            AppColors.accentPrimary.opacity(0.25),
                            AppColors.accentPrimary.opacity(0.04),
                        ],
                        center: .center,
                        startRadius: 5,
                        endRadius: 55
                    )
                )
                .frame(width: 100, height: 100)

            Image(systemName: "mic.fill")
                .font(.system(size: 36, weight: .light))
                .foregroundColor(.white.opacity(0.75))
        }
    }

    // MARK: - Start Recording button

    private var startButton: some View {
        Button(action: onStartRecording) {
            HStack(spacing: 8) {
                Image(systemName: "mic.fill")
                    .font(.system(size: 12))
                Text("Start Recording")
                    .font(.system(size: 13, weight: .semibold))
            }
            .foregroundColor(.white)
            .padding(.horizontal, 22)
            .padding(.vertical, 10)
            .background(
                Capsule()
                    .fill(
                                LinearGradient(
                                    colors: [
                                        AppColors.accentPrimary,
                                        AppColors.accentSecondary,
                                    ],
                            startPoint: .leading,
                            endPoint: .trailing
                        )
                    )
            )
            .shadow(
                color: AppColors.accentPrimary.opacity(isButtonHovered ? 0.45 : 0.0),
                radius: isButtonHovered ? 12 : 0
            )
            .scaleEffect(isButtonHovered ? 1.04 : 1.0)
            .animation(.easeOut(duration: 0.15), value: isButtonHovered)
        }
        .buttonStyle(.plain)
        .onHover { isButtonHovered = $0 }
    }
}
