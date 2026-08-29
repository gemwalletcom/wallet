// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSupportStore
import typealias Gemstone.SupportMessage
import typealias Gemstone.SupportTyping
import GemstonePrimitives
import Primitives
import Store

@Observable
public final class GemstoneSupportStore: GemSupportStore, @unchecked Sendable {
    @ObservationIgnored
    private let store: SupportChatStore
    @ObservationIgnored
    private let lock = NSLock()
    @ObservationIgnored
    private var storedAgent: SupportAgent?

    public init(store: SupportChatStore) {
        self.store = store
    }

    public var typingAgent: SupportAgent? {
        access(keyPath: \.typingAgent)
        return lock.withLock { storedAgent }
    }

    public func saveMessages(messages: [Gemstone.SupportMessage]) async throws {
        try store.addMessages(messages.map { try Primitives.SupportMessage($0) })
    }

    public func saveMessage(id: String, message: Gemstone.SupportMessage) async throws {
        try store.replace(id: id, with: Primitives.SupportMessage(message))
    }

    public func updateTyping(typing: Gemstone.SupportTyping) throws {
        let typing = try Primitives.SupportTyping(typing)
        switch typing.status {
        case .on: setTypingAgent(typing.agent)
        case .off: setTypingAgent(.none)
        }
    }

    public func clearTyping() throws {
        setTypingAgent(.none)
    }

    public func clearTypingAgent() {
        setTypingAgent(.none)
    }

    private func setTypingAgent(_ agent: SupportAgent?) {
        withMutation(keyPath: \.typingAgent) {
            lock.withLock { storedAgent = agent }
        }
    }
}
