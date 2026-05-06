import SwiftUI

struct GlassCardView<Content: View>: View {
    let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        content
            .background(
                RoundedRectangle(cornerRadius: 20)
                    .fill(Color(white: 0.08).opacity(0.92))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 20)
                    .stroke(
                        LinearGradient(
                            colors: [
                                Color.white.opacity(0.18),
                                Color.white.opacity(0.06),
                                Color.white.opacity(0.02),
                            ],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        ),
                        lineWidth: 0.5
                    )
            )
            .clipShape(RoundedRectangle(cornerRadius: 20))
    }
}

#if DEBUG
#Preview("GlassCardView") {
    GlassCardView {
        VStack(spacing: 8) {
            Text("Sample content")
                .foregroundColor(.white)
            Text("Preview")
                .font(.caption)
                .foregroundColor(.white.opacity(0.7))
        }
        .padding(24)
    }
    .frame(width: 200, height: 120)
}
#endif
