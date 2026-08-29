// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSupportServiceProtocol
import GemstonePrimitives
import Localization
import PhotosUI
import Primitives
import Store
import GemstoneServices
import SwiftUI

@Observable
@MainActor
public final class SupportChatSceneViewModel {
    private let service: any GemSupportServiceProtocol
    private let typing: SupportTypingState
    public let query: ObservableQuery<SupportMessagesRequest>
    var previewURL: URL?

    public init(service: any GemSupportServiceProtocol, typing: SupportTypingState) {
        self.service = service
        self.typing = typing
        query = ObservableQuery(SupportMessagesRequest(), initialValue: [])
    }

    var title: String { Localized.Settings.support }
    var emptyTitle: String { Localized.Support.stateEmptyTitle }
    var emptyDescription: String { Localized.Support.stateEmptyDescription }
    var isEmpty: Bool { query.value.isEmpty }
    var typingAgentName: String? { typing.agent?.name }

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

    func load() async {
        let fromTimestamp = query.value.last { $0.sender.isAgent }.map { Int($0.createdAt.timeIntervalSince1970) } ?? 0
        await perform("load") {
            try await service.syncMessages(fromTimestamp: fromTimestamp)
        }
    }

    func onScenePhaseChange(_: ScenePhase, _ newPhase: ScenePhase) {
        switch newPhase {
        case .active: Task { await load() }
        case .inactive, .background: break
        @unknown default: break
        }
    }

    func onDisappear() {
        typing.clear()
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
                previewURL = URL(fileURLWithPath: try await service.imageFile(url: url.absoluteString))
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
