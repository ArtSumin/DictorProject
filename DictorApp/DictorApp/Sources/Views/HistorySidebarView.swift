import SwiftUI
import AppKit

public struct HistoryEntry: Identifiable {
    public let id: UUID
    public let text: String
    public let date: Date
    /// Database id for delete operations. Nil for newly transcribed items not yet loaded from DB.
    public let dbId: Int64?

    public var previewTitle: String {
        String(text.prefix(40))
        + (text.count > 40 ? "…" : "")
    }

    public init(id: UUID = UUID(), text: String, date: Date = Date(), dbId: Int64? = nil) {
        self.id = id
        self.text = text
        self.date = date
        self.dbId = dbId
    }
}

struct HistorySidebarView: View {
    @EnvironmentObject var viewModel: RecorderViewModel
    @State private var copiedItemId: UUID?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                ForEach(viewModel.historyEntries) { item in
                    rowContent(for: item)
                }
            }
            .animation(.easeInOut(duration: 0.25), value: viewModel.historyEntries.map(\.id))
        }
        .frame(maxHeight: 160)
        .padding(.vertical, 6)
        .animation(.easeInOut(duration: 0.2), value: copiedItemId)
    }

    @ViewBuilder
    private func rowContent(for item: HistoryEntry) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(item.previewTitle)
                    .font(.system(size: 12))
                    .foregroundColor(.white.opacity(0.7))
                    .lineLimit(1)
                Spacer()
                if copiedItemId == item.id {
                    Text("Copied")
                        .font(.system(size: 10))
                        .foregroundColor(.white.opacity(0.5))
                        .transition(.opacity.combined(with: .scale(scale: 0.8)))
                }
                if let dbId = item.dbId {
                    Button(action: {
                        viewModel.deleteHistoryEntry(dbId: dbId)
                    }) {
                        Image(systemName: "trash")
                            .font(.system(size: 10))
                            .foregroundColor(.white.opacity(0.35))
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .contentShape(Rectangle())
            .transition(.opacity.combined(with: .move(edge: .trailing)))
            .onTapGesture {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(item.text, forType: .string)
                withAnimation(.easeInOut(duration: 0.2)) {
                    copiedItemId = item.id
                }
                Task { @MainActor in
                    try? await Task.sleep(for: .seconds(1.5))
                    withAnimation(.easeInOut(duration: 0.2)) {
                        copiedItemId = nil
                    }
                }
            }

            if item.id != viewModel.historyEntries.last?.id {
                Rectangle()
                    .fill(Color.white.opacity(0.04))
                    .frame(height: 1)
                    .padding(.horizontal, 16)
            }
        }
        .transition(.opacity.combined(with: .move(edge: .trailing)))
    }
}
