// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemLoadState
import protocol Gemstone.GemNotificationsServiceProtocol
import protocol Gemstone.GemSupportServiceProtocol
import func Gemstone.loadError
import GemstonePrimitives
import GemstoneServices
import Localization
import PhotosUI
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class SupportChatSceneViewModel {
    private let service: any GemSupportServiceProtocol
    private let notifications: any GemNotificationsServiceProtocol
    private let typing: ObservableSupportTyping
    public let query: ObservableQuery<SupportMessagesQuery>
    var previewURL: URL?
    var isPresentingAlertMessage: AlertMessage?

    private var loadState: GemLoadState = .loading

    public init(service: any GemSupportServiceProtocol, notifications: any GemNotificationsServiceProtocol, typing: ObservableSupportTyping) {
        self.service = service
        self.notifications = notifications
        self.typing = typing
        query = ObservableQuery(SupportMessagesQuery(), initialValue: [])
    }

    var title: String { Localized.Settings.support }
    var emptyTitle: String { Localized.Support.stateEmptyTitle }
    var emptyDescription: String { Localized.Support.stateEmptyDescription }
    var isEmpty: Bool { query.value.isEmpty }

    var loadError: Error? {
        Gemstone.loadError(state: loadState, hasRows: !isEmpty)
    }

    var typingAgentName: String? { typing.agent?.name }

    @ObservationIgnored
    private(set) lazy var inputBarModel = SupportMessageInputBarViewModel(
        onSendText: { [weak self] in self?.onSendText($0) },
        onSendImages: { [weak self] in self?.onSendImages($0) },
    )

    var days: [SupportChatDay] {
        SupportChatDayBuilder(messages: query.value).build()
    }

    func load() async {
        let fromTimestamp = service.syncFromTimestamp(messages: query.value.map { $0.toGem() })
        loadState = await service.refresh(fromTimestamp: fromTimestamp, hasMessages: !isEmpty)
    }

    func enableNotificationsForSupport() async {
        guard case let .notRegistered(error) = await notifications.askToEnable()?.result else { return }
        isPresentingAlertMessage = AlertMessage(message: error.text)
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

    func sendText(_ content: String) async {
        await alertOnFailure {
            try await service.sendMessage(.text(content))
        }
    }

    func sendImages(_ items: [PhotosPickerItem]) async {
        for item in items {
            await alertOnFailure {
                guard let attachment = try await item.imageAttachment() else {
                    throw AnyError(Localized.Errors.notSupported)
                }
                try await service.sendMessage(.image(attachment))
            }
        }
    }

    func retry(_ message: SupportMessage) async {
        await alertOnFailure {
            try await service.retryMessage(message)
        }
    }

    func openPreview(_ image: SupportMessageImage) async {
        guard let url = image.url.asURL else { return }
        await alertOnFailure {
            previewURL = try await URL(fileURLWithPath: service.imageFile(url: url.absoluteString))
        }
    }
}

// MARK: - Actions

extension SupportChatSceneViewModel {
    func onRetry(_ message: SupportMessage) {
        Task { await retry(message) }
    }

    func onOpenPreview(_ image: SupportMessageImage) {
        Task { await openPreview(image) }
    }
}

// MARK: - Private

private extension SupportChatSceneViewModel {
    func onSendText(_ content: String) {
        Task { await sendText(content) }
    }

    func onSendImages(_ items: [PhotosPickerItem]) {
        Task { await sendImages(items) }
    }

    func alertOnFailure(_ operation: () async throws -> Void) async {
        do {
            try await operation()
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}
