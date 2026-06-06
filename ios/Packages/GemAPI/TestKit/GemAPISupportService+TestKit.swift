// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public actor GemAPISupportServiceMock: GemAPISupportService {
    private let messages: [SupportMessage]

    public private(set) var sentMessages: [SupportMessageInput] = []
    public private(set) var sentImages: [(image: Data, fileName: String, mimeType: String)] = []
    public private(set) var sentActions: [SupportAction] = []

    public init(
        messages: [SupportMessage] = [],
    ) {
        self.messages = messages
    }

    public func getSupportMessages(fromTimestamp _: Int) async throws -> [SupportMessage] {
        messages
    }

    public func sendSupportMessage(input: SupportMessageInput) async throws -> SupportMessage {
        sentMessages.append(input)
        return SupportMessage(
            id: "",
            content: input.content,
            sender: .user,
            status: .sent,
            createdAt: Date(),
            images: [],
        )
    }

    public func sendSupportImage(image: Data, fileName: String, mimeType: String) async throws -> SupportMessage {
        sentImages.append((image, fileName, mimeType))
        return SupportMessage(
            id: "",
            content: "",
            sender: .user,
            status: .sent,
            createdAt: Date(),
            images: [],
        )
    }

    public func sendSupportAction(action: SupportAction) async throws {
        sentActions.append(action)
    }
}
