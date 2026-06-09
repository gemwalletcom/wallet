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

    public init(service: SupportChatService) {
        self.service = service
        query = ObservableQuery(SupportMessagesRequest(), initialValue: [])
    }

    var title: String { Localized.Settings.support }
    var emptyTitle: String { Localized.Support.stateEmptyTitle }
    var emptyDescription: String { Localized.Support.stateEmptyDescription }

    private var seenAgentCount = 0
    var isAtBottom = true
    var previewURLs: [URL] = []
    var isPresentingImagePreview: URL?
    var isEmpty: Bool { query.value.isEmpty }
    var unreadAgentCount: Int { max(0, agentCount - seenAgentCount) }
    var agentCount: Int { query.value.filter { $0.sender.isAgent }.count }

    var inputBarModel: SupportMessageInputBarViewModel {
        SupportMessageInputBarViewModel(
            onSendText: { [weak self] in self?.sendText($0) },
            onSendImages: { [weak self] in self?.sendImages($0) },
        )
    }

    var days: [SupportChatDay] {
        SupportChatDayBuilder(
            messages: query.value,
            retryAction: { [weak self] in self?.onRetry($0) },
            imageAction: { [weak self] request in Task { await self?.openPreview(request) } },
            displayURL: service.displayURL(for:),
        ).build()
    }

    func setAtBottom(_ atBottom: Bool) {
        isAtBottom = atBottom
        if atBottom { seenAgentCount = agentCount }
    }

    func openPreview(_ request: SupportImagePreviewRequest) async {
        do {
            let urls = try await service.previewFileURLs(for: request.images)
            previewURLs = urls
            isPresentingImagePreview = request.images.firstIndex { $0.id == request.selectedId }.map { urls[$0] } ?? urls.first
        } catch {
            debugLog("SupportChatSceneViewModel preview error: \(error)")
        }
    }

    func fetch() async {
        do {
            let fromTimestamp = query.value.last { $0.sender.isAgent }.map { Int($0.createdAt.timeIntervalSince1970) } ?? 0
            try await service.syncMessages(fromTimestamp: fromTimestamp)
        } catch {
            debugLog("SupportChatSceneViewModel fetch error: \(error)")
        }
    }

    func sendText(_ content: String) {
        Task {
            do {
                try await service.sendText(content)
            } catch {
                debugLog("SupportChatSceneViewModel send text error: \(error)")
            }
        }
    }

    func sendImages(_ items: [PhotosPickerItem]) {
        Task {
            var attachments: [ImageAttachment] = []
            for item in items {
                guard let attachment = try? await item.imageAttachment() else { continue }
                attachments.append(attachment)
            }
            guard !attachments.isEmpty else { return }
            do {
                try await service.sendImages(attachments)
            } catch {
                debugLog("SupportChatSceneViewModel send images error: \(error)")
            }
        }
    }

    func onRetry(_ message: SupportMessage) {
        Task {
            do {
                try await service.retryMessage(message)
            } catch {
                debugLog("SupportChatSceneViewModel retry error: \(error)")
            }
        }
    }
}
