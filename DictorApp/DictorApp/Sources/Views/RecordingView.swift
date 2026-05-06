import SwiftUI

struct RecordingView: View {
    var onStopRecording: () -> Void
    /// Pass real-time dB for reactive waveform; nil falls back to sine animation.
    var audioLevelDecibels: Float? = nil

    @State private var isButtonHovered = false

    var body: some View {
        VStack(spacing: 0) {
            Spacer()

            WaveformView(audioLevelDecibels: audioLevelDecibels)
                .frame(height: 90)
                .padding(.horizontal, 24)

            Spacer()

            stopButton
                .padding(.bottom, 8)
        }
    }

    private var stopButton: some View {
        Button(action: onStopRecording) {
            HStack(spacing: 8) {
                RoundedRectangle(cornerRadius: 2)
                    .fill(Color.white)
                    .frame(width: 11, height: 11)
                Text("Stop Recording")
                    .font(.system(size: 13, weight: .semibold))
            }
            .foregroundColor(.white)
            .padding(.horizontal, 22)
            .padding(.vertical, 10)
            .background(
                Capsule()
                    .fill(AppColors.accentDestructive)
            )
            .shadow(
                color: AppColors.accentDestructive.opacity(isButtonHovered ? 0.5 : 0.0),
                radius: isButtonHovered ? 12 : 0
            )
            .scaleEffect(isButtonHovered ? 1.04 : 1.0)
            .animation(.easeOut(duration: 0.15), value: isButtonHovered)
        }
        .buttonStyle(.plain)
        .onHover { isButtonHovered = $0 }
    }
}
