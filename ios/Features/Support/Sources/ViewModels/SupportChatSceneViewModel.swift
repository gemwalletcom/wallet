// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import PhotosUI
import Primitives
import Store
import SupportChatService
import SwiftUI

@Observable
@MainActor
public final class SupportChatSceneViewModel {
    private let service: SupportChatService
    public let query: ObservableQuery<SupportMessagesRequest>
    var previewURL: URL?

    public init(service: SupportChatService) {
        self.service = service
        query = ObservableQuery(SupportMessagesRequest(), initialValue: [])
    }

    var title: String { Localized.Settings.support }
    var emptyTitle: String { Localized.Support.stateEmptyTitle }
    var emptyDescription: String { Localized.Support.stateEmptyDescription }
    var isEmpty: Bool { query.value.isEmpty }

    @ObservationIgnored
    private(set) lazy var inputBarModel = SupportMessageInputBarViewModel(
        onSendText: { [weak self] in self?.sendText($0) },
        onSendImages: { [weak self] in self?.sendImages($0) },
    )

    var days: [SupportChatDay] {
        SupportChatDayBuilder(
            messages: query.value,
            retryAction: { [weak self] in self?.retry($0) },
            imageAction: { [weak self] in self?.openPreview($0) },
        ).build()
    }

    func fetch() async {
        let fromTimestamp = query.value.last { $0.sender.isAgent }.map { Int($0.createdAt.timeIntervalSince1970) } ?? 0
        await perform("fetch") {
            try await service.syncMessages(fromTimestamp: fromTimestamp)
        }
    }

    func sendText(_ content: String) {
        Task {
            await perform("send text") {
                try await service.sendMessage(.text(content))
            }
        }
    }

    func sendImages(_ items: [PhotosPickerItem]) {
        Task {
            for item in items {
                guard let attachment = try? await item.imageAttachment() else { continue }
                await perform("send image") {
                    try await service.sendMessage(.image(attachment))
                }
            }
        }
    }

    func retry(_ message: SupportMessage) {
        Task {
            await perform("retry") {
                try await service.retryMessage(message)
            }
        }
    }

    func openPreview(_ image: SupportMessageImage) {
        guard let url = image.url.asURL else { return }
        Task {
            await perform("preview") {
                previewURL = try await service.imageFile(for: url)
            }
        }
    }
}

// MARK: - Private

private extension SupportChatSceneViewModel {
    func perform(_ context: String, _ operation: () async throws -> Void) async {
        do {
            try await operation()
        } catch {
            debugLog("SupportChatSceneViewModel \(context) error: \(error)")
        }
    }
}
