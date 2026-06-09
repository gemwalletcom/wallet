// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct SupportChatDayBuilder {
    let messages: [SupportMessage]
    let retryAction: (SupportMessage) -> Void
    let imageAction: (SupportImagePreviewRequest) -> Void
    let displayURL: (SupportMessageImage) -> URL?

    func build() -> [SupportChatDay] {
        let sortedDays = Dictionary(grouping: messages) { Calendar.current.startOfDay(for: $0.createdAt) }
            .sorted { $0.key < $1.key }
        return sortedDays.enumerated().map { index, day in
            SupportChatDay(
                id: day.key.ISO8601Format(),
                date: day.key,
                groups: groups(from: day.value, isLastDay: index == sortedDays.indices.last),
            )
        }
    }
}

// MARK: - Private

private extension SupportChatDayBuilder {
    func groups(from messages: [SupportMessage], isLastDay: Bool) -> [SupportChatGroup] {
        let kinds = messages.chunked(on: senderKey).compactMap(kind(from:))
        return kinds.enumerated().map { index, kind in
            SupportChatGroup(kind: kind, isLast: isLastDay && index == kinds.indices.last)
        }
    }

    func kind(from messages: [SupportMessage]) -> SupportChatGroup.Kind? {
        guard let sender = messages.first?.sender else { return nil }
        let bubbles = messages.map { SupportMessageBubbleViewModel(message: $0, retryAction: retryAction, imageAction: imageAction, displayURL: displayURL) }
        switch sender {
        case .user: return .user(messages: bubbles)
        case let .agent(agent): return .agent(header: SupportAgentHeader(agent: agent), messages: bubbles)
        }
    }

    func senderKey(_ message: SupportMessage) -> String {
        switch message.sender {
        case .user: "user"
        case let .agent(agent): "agent-\(agent.name)"
        }
    }
}

private extension Array {
    func chunked(on key: (Element) -> some Equatable) -> [[Element]] {
        var chunks: [[Element]] = []
        var currentChunk: [Element] = []
        for element in self {
            if let last = currentChunk.last, key(last) != key(element) {
                chunks.append(currentChunk)
                currentChunk = []
            }
            currentChunk.append(element)
        }
        if currentChunk.isNotEmpty {
            chunks.append(currentChunk)
        }
        return chunks
    }
}
