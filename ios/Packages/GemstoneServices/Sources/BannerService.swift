// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemBannerService
import GemstonePrimitives
import Primitives
import Store
import UIKit

public struct BannerService: Sendable {
    private let store: BannerStore
    private let service: GemBannerService
    private let pushNotificationService: PushNotificationEnablerService

    public init(
        store: BannerStore,
        service: GemBannerService,
        pushNotificationService: PushNotificationEnablerService,
    ) {
        self.store = store
        self.service = service
        self.pushNotificationService = pushNotificationService
    }

    public func handleAction(_ action: BannerAction) async throws {
        let canClose = switch action.type {
        case let .event(event):
            if service.closesOnAction(event: try event.json()) {
                try await pushNotificationService.requestPermissionsOrOpenSettings()
            } else {
                false
            }
        case .closeBanner: true
        case .button: false
        }
        if canClose {
            try closeBanner(id: action.id)
        }
    }

    @discardableResult
    public func clearBanners() throws -> Int {
        try store.clear()
    }

    @discardableResult
    public func activateAllCancelledBanners() throws -> Int {
        try store.updateStates(from: .cancelled, to: .active)
    }

    private func updateState(id: String, state: BannerState) throws {
        try store.updateState(id, state: state)
    }
}

// MARK: - Actions

public extension BannerService {
    func closeBanner(id: String) throws {
        try updateState(id: id, state: .cancelled)
    }

    func onClose(_ banner: Banner) {
        Task { try closeBanner(id: banner.id) }
    }
}
