// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSupportServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct SupportChatService: Sendable {
    private let store: SupportChatStore
    private let provider: any GemSupportServiceProtocol
    public let typing: SupportTypingState

    public init(
        store: SupportChatStore,
        provider: any GemSupportServiceProtocol,
        typing: SupportTypingState,
    ) {
        self.store = store
        self.provider = provider
        self.typing = typing
    }

    public func receive(_ event: SupportStreamEvent) async throws {
        switch event {
        case let .message(message):
            try store.addMessages([message])
            switch message.sender {
            case .user: break
            case .agent: await typing.clear()
            }
        case let .typing(payload):
            await typing.update(payload)
        }
    }

    public func syncMessages(fromTimestamp: Int) async throws {
        try await provider.syncMessages(fromTimestamp: UInt64(fromTimestamp))
    }

    public func sendMessage(_ content: SupportMessageContent) async throws {
        switch content {
        case let .text(text):
            try await provider.sendText(content: text)
        case let .image(attachment):
            try await provider.sendImage(image: attachment.data, fileName: attachment.fileName, mimeType: attachment.mimeType)
        }
    }

    public func retryMessage(_ message: SupportMessage) async throws {
        try await provider.retryMessage(message: message.json())
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
