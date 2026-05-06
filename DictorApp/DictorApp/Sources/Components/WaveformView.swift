import SwiftUI

struct WaveformView: View {
    var compact = false
    /// dBFS (-160…0). If nil, falls back to sine wave animation (e.g. for previews).
    var audioLevelDecibels: Float? = nil

    private var barCount: Int { compact ? 30 : 40 }
    private var barWidth: CGFloat { compact ? 2 : 3 }
    private var barSpacing: CGFloat { compact ? 1.5 : 2.5 }
    private var maxBarHeight: CGFloat { compact ? 20 : 80 }
    private let minBarHeight: CGFloat = 3

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 60.0)) { timeline in
            let time = timeline.date.timeIntervalSinceReferenceDate
            HStack(alignment: .center, spacing: barSpacing) {
                ForEach(0..<barCount, id: \.self) { index in
                    waveformBar(index: index, time: time)
                }
            }
            .animation(.easeOut(duration: 0.04), value: audioLevelDecibels ?? -160)
        }
    }

    private func waveformBar(index: Int, time: Double) -> some View {
        let center = Double(barCount) / 2.0
        let distance = abs(Double(index) - center) / center
        let envelope = max(0.08, 1.0 - distance * distance)

        let height: CGFloat
        if let db = audioLevelDecibels {
            // Диапазон -42…0 dB — умеренная чувствительность, меньше реакции на шум и тихий фон
            let rawNormalized = max(0, min(1, Double(db + 42) / 42))
            // Power curve (0.65) — менее чувствительно к тихим звукам, требуется громче речь
            let curved = pow(rawNormalized, 0.65)
            // Усиление ×1.4 — полоски реагируют мягче
            let boosted = min(1, curved * 1.4)
            // Две волны — лёгкое «дыхание» анимации
            let life1 = sin(time * 4.5 + Double(index) * 0.4) * 0.08
            let life2 = sin(time * 2.8 + Double(index) * 0.15) * 0.04
            let life = life1 + life2
            let finalNormalized = max(0, min(1, boosted + life))
            let range = maxBarHeight - minBarHeight
            height = minBarHeight + CGFloat(finalNormalized) * range * envelope
        } else {
            let phase = Double(index) * 0.25
            let wave1 = sin(time * 3.5 + phase) * 0.35
            let wave2 = sin(time * 5.8 + phase * 1.3) * 0.25
            let wave3 = sin(time * 8.2 + phase * 0.7) * 0.15
            let combined = 0.5 + wave1 + wave2 + wave3
            height = max(minBarHeight, CGFloat(envelope * max(0.08, combined)) * maxBarHeight)
        }

        return RoundedRectangle(cornerRadius: 1.5)
            .fill(
                LinearGradient(
                    colors: [
                        Color.cyan.opacity(0.9),
                        Color.blue,
                        Color.purple.opacity(0.9),
                    ],
                    startPoint: .bottom,
                    endPoint: .top
                )
            )
            .frame(width: barWidth, height: height)
    }
}
