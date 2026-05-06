import SwiftUI

struct ProcessingView: View {

    var body: some View {
        VStack(spacing: 16) {
            Spacer()

            ProgressView()
                .progressViewStyle(.circular)
                .scaleEffect(1.4)
                .tint(.white.opacity(0.6))

            Text("Processing text...")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white.opacity(0.5))

            Spacer()

            disabledStopButton
                .padding(.bottom, 8)
        }
    }

    private var disabledStopButton: some View {
        HStack(spacing: 8) {
            RoundedRectangle(cornerRadius: 2)
                .fill(Color.white.opacity(0.35))
                .frame(width: 11, height: 11)
            Text("Stop Recording")
                .font(.system(size: 13, weight: .semibold))
        }
        .foregroundColor(.white.opacity(0.35))
        .padding(.horizontal, 22)
        .padding(.vertical, 10)
        .background(
            Capsule()
                .fill(Color.white.opacity(0.08))
        )
    }
}
