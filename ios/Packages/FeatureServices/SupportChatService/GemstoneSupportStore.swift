// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSupportStore
import typealias Gemstone.SupportMessage
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneSupportStore: GemSupportStore, @unchecked Sendable {
    private let store: SupportChatStore

    public init(store: SupportChatStore) {
        self.store = store
    }

    public func saveMessages(messages: [Gemstone.SupportMessage]) async throws {
        try store.addMessages(messages.map { try Primitives.SupportMessage($0) })
    }

    public func replaceMessage(id: String, message: Gemstone.SupportMessage) async throws {
        try store.replace(id: id, with: Primitives.SupportMessage(message))
    }
}
