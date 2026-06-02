// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public actor GemAPISupportServiceMock: GemAPISupportService {
    private let conversation: SupportConversation?
    private let messages: [SupportMessage]

    public private(set) var sentMessages: [SupportMessageInput] = []
    public private(set) var sentImages: [(image: Data, fileName: String, mimeType: String)] = []
    public private(set) var sentActions: [SupportAction] = []

    public init(
        conversation: SupportConversation? = nil,
        messages: [SupportMessage] = [],
    ) {
        self.conversation = conversation
        self.messages = messages
    }

    public func getSupportConversation() async throws -> SupportConversation? {
        conversation
    }

    public func getSupportMessages(fromTimestamp _: Int) async throws -> [SupportMessage] {
        messages
    }

    public func sendSupportMessage(input: SupportMessageInput) async throws -> SupportMessage {
        sentMessages.append(input)
        return SupportMessage(
            id: "",
            conversationId: "",
            content: input.content,
            sender: .user,
            deliveryStatus: .sent,
            createdAt: Date(),
            images: [],
        )
    }

    public func sendSupportImage(image: Data, fileName: String, mimeType: String) async throws -> SupportMessage {
        sentImages.append((image, fileName, mimeType))
        return SupportMessage(
            id: "",
            conversationId: "",
            content: "",
            sender: .user,
            deliveryStatus: .sent,
            createdAt: Date(),
            images: [],
        )
    }

    public func sendSupportAction(action: SupportAction) async throws {
        sentActions.append(action)
    }
}
