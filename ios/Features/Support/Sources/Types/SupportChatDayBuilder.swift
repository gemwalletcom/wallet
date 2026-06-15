// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct SupportChatDayBuilder {
    let messages: [SupportMessage]
    let retryAction: (SupportMessage) -> Void
    let imageAction: (SupportMessageImage) -> Void

    func build() -> [SupportChatDay] {
        Dictionary(grouping: messages) { Calendar.current.startOfDay(for: $0.createdAt) }
            .sorted { $0.key < $1.key }
            .map { day in
                SupportChatDay(date: day.key, groups: groups(from: day.value))
            }
    }
}

// MARK: - Private

private extension SupportChatDayBuilder {
    func groups(from messages: [SupportMessage]) -> [SupportChatGroup] {
        messages.chunked(on: senderKey)
            .compactMap(kind(from:))
            .map { SupportChatGroup(kind: $0) }
    }

    func kind(from messages: [SupportMessage]) -> SupportChatGroup.Kind? {
        guard let sender = messages.first?.sender else { return nil }
        let bubbles = messages.map { SupportMessageBubbleViewModel(message: $0, retryAction: retryAction, imageAction: imageAction) }
        switch sender {
        case .user: return .user(messages: bubbles)
        case let .agent(agent): return .agent(name: agent.name, messages: bubbles)
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
