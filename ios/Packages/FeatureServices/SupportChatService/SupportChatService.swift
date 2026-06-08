// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import Store

public final class SupportChatService: Sendable {
    private let store: SupportChatStore
    private let provider: any GemAPISupportService
    private let imageStore = SupportImageStore()

    public init(
        store: SupportChatStore,
        provider: any GemAPISupportService = GemAPIService.shared,
    ) {
        self.store = store
        self.provider = provider
    }

    public func syncMessages(fromTimestamp: Int) async throws {
        try store.addMessages(await provider.getSupportMessages(fromTimestamp: fromTimestamp))
    }

    public func sendText(_ content: String) async throws {
        let message = SupportMessage.userText(content)
        try store.addMessages([message])
        await deliver(message)
    }

    public func sendImages(_ attachments: [ImageAttachment]) async throws {
        let messages = try attachments.map(pendingImageMessage)
        try store.addMessages(messages)
        for message in messages {
            await deliver(message)
        }
    }

    public func retryMessage(_ message: SupportMessage) async throws {
        try store.addMessages([message.with(status: .sending)])
        await deliver(message)
    }
}

// MARK: - Private

private extension SupportChatService {
    func pendingImageMessage(_ attachment: ImageAttachment) throws -> SupportMessage {
        let id = UUID().uuidString
        let url = try imageStore.store(attachment.data, id: id)
        return .userImage(id: id, url: url, fileName: attachment.fileName, fileSize: attachment.data.count)
    }

    func deliver(_ message: SupportMessage) async {
        do {
            let sent = try await send(message)
            try store.replace(id: message.id, with: sent.with(images: message.images))
        } catch {
            try? store.addMessages([message.with(status: .failed)])
        }
    }

    func send(_ message: SupportMessage) async throws -> SupportMessage {
        guard let image = message.images.first else {
            return try await provider.sendSupportMessage(input: SupportMessageInput(content: message.content))
        }
        guard let url = image.url.asURL, let data = imageStore.data(at: url) else {
            throw SupportChatError.imageDataUnavailable
        }
        return try await provider.sendSupportImage(image: data, fileName: image.fileName ?? "image", mimeType: image.mimeType)
    }
}
