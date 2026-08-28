// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSupportStore
import typealias Gemstone.SupportMessage
import typealias Gemstone.SupportTyping
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneSupportStore: GemSupportStore, @unchecked Sendable {
    private let store: SupportChatStore
    private let typing: SupportTypingState

    public init(store: SupportChatStore, typing: SupportTypingState) {
        self.store = store
        self.typing = typing
    }

    public func saveMessages(messages: [Gemstone.SupportMessage]) async throws {
        try store.addMessages(messages.map { try Primitives.SupportMessage($0) })
    }

    public func saveMessage(id: String, message: Gemstone.SupportMessage) async throws {
        try store.replace(id: id, with: Primitives.SupportMessage(message))
    }

    public func updateTyping(typing: Gemstone.SupportTyping) throws {
        let typing = try Primitives.SupportTyping(typing)
        Task { @MainActor in self.typing.update(typing) }
    }

    public func clearTyping() throws {
        Task { @MainActor in typing.clear() }
    }
}
