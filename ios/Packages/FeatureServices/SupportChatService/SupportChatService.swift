// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import Store

public final class SupportChatService: Sendable {
    private let store: SupportChatStore
    private let provider: any GemAPISupportService
    private let uploadStore = SupportImageStore(.uploads)
    private let previewStore = SupportImageStore(.previews)

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

    public func displayURL(for image: SupportMessageImage) -> URL? {
        uploadStore.file(id: image.id, fileExtension: image.fileExtension) ?? image.url.asURL
    }

    public func previewFileURLs(for images: [SupportMessageImage]) async throws -> [URL] {
        var urls: [URL] = []
        for image in images {
            try await urls.append(previewFileURL(for: image))
        }
        return urls
    }
}

// MARK: - Private

private extension SupportChatService {
    func previewFileURL(for image: SupportMessageImage) async throws -> URL {
        if let local = uploadStore.file(id: image.id, fileExtension: image.fileExtension) {
            return local
        }
        if let cached = previewStore.file(id: image.id, fileExtension: image.fileExtension) {
            return cached
        }
        guard let url = image.url.asURL else { throw SupportChatError.imageDataUnavailable }
        let data = try await loadData(from: url)
        return try previewStore.store(data, id: image.id, fileExtension: image.fileExtension)
    }

    func pendingImageMessage(_ attachment: ImageAttachment) throws -> SupportMessage {
        let id = UUID().uuidString
        let fileExtension = (attachment.fileName as NSString).pathExtension
        let url = try uploadStore.store(attachment.data, id: id, fileExtension: fileExtension.isEmpty ? "jpg" : fileExtension)
        return .userImage(id: id, url: url, fileName: attachment.fileName, fileSize: attachment.data.count)
    }

    func deliver(_ message: SupportMessage) async {
        do {
            let sent = try await send(message)
            let images = zip(message.images, sent.images).map { $0.with(url: $1.url) }
            try store.replace(id: message.id, with: sent.with(images: images))
        } catch {
            try? store.addMessages([message.with(status: .failed)])
        }
    }

    func send(_ message: SupportMessage) async throws -> SupportMessage {
        guard let image = message.images.first else {
            return try await provider.sendSupportMessage(input: SupportMessageInput(content: message.content))
        }
        guard let url = image.url.asURL, let data = uploadStore.data(at: url) else {
            throw SupportChatError.imageDataUnavailable
        }
        return try await provider.sendSupportImage(image: data, fileName: image.fileName ?? "image", mimeType: image.mimeType)
    }

    func loadData(from url: URL) async throws -> Data {
        let request = URLRequest(url: url)
        if let cached = URLCache.shared.cachedResponse(for: request)?.data {
            return cached
        }
        let (data, response) = try await URLSession.shared.data(for: request)
        if let http = response as? HTTPURLResponse, !(200 ..< 300).contains(http.statusCode) {
            throw SupportChatError.imageDataUnavailable
        }
        URLCache.shared.storeCachedResponse(CachedURLResponse(response: response, data: data), for: request)
        return data
    }
}
