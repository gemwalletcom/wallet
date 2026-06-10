// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import Store

public struct SupportChatService: Sendable {
    private let store: SupportChatStore
    private let provider: any GemAPISupportService

    public init(
        store: SupportChatStore,
        provider: any GemAPISupportService = GemAPIService.shared,
    ) {
        self.store = store
        self.provider = provider
    }

    public func syncMessages(fromTimestamp: Int) async throws {
        try await store.addMessages(provider.getSupportMessages(fromTimestamp: fromTimestamp))
    }

    public func sendMessage(_ content: SupportMessageContent) async throws {
        let message = SupportMessage.pending(content)
        try store.addMessages([message])
        await deliver(message, content: content)
    }

    public func retryMessage(_ message: SupportMessage) async throws {
        try store.addMessages([message.with(status: .sending)])
        await deliver(message, content: .text(message.content))
    }

    public func imageFile(for url: URL) async throws -> URL {
        let request = URLRequest(url: url)
        let data = if let cached = URLCache.shared.cachedResponse(for: request)?.data {
            cached
        } else {
            try await URLSession.shared.data(for: request).0
        }
        let file = FileManager.default.temporaryDirectory.appendingPathComponent(url.lastPathComponent)
        try data.write(to: file)
        return file
    }
}

// MARK: - Private

private extension SupportChatService {
    func deliver(_ message: SupportMessage, content: SupportMessageContent) async {
        do {
            let sent = try await send(content)
            try store.replace(id: message.id, with: sent)
        } catch {
            try? store.addMessages([message.with(status: .failed)])
        }
    }

    func send(_ content: SupportMessageContent) async throws -> SupportMessage {
        switch content {
        case let .text(text):
            try await provider.sendSupportMessage(input: SupportMessageInput(content: text))
        case let .image(attachment):
            try await provider.sendSupportImage(image: attachment.data, fileName: attachment.fileName, mimeType: attachment.mimeType)
        }
    }
}
