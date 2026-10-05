// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListPhase
import enum Gemstone.GemLoadState
import enum Gemstone.GemNotificationDestination
import struct Gemstone.GemNotificationRow
import protocol Gemstone.GemNotificationServiceProtocol
import func Gemstone.notificationListPhase
import func Gemstone.notificationRows
import enum Gemstone.UrlAction
import Localization
import Primitives
import PrimitivesComponents
import Store
import UIKit

@Observable
@MainActor
public final class InAppNotificationsSceneViewModel {
    private let service: any GemNotificationServiceProtocol
    private let onOpenAction: ((UrlAction) -> Void)?

    private var loadState: GemLoadState = .loading

    public let query: ObservableQuery<InAppNotificationsQuery>
    public var notifications: [Primitives.InAppNotification] {
        query.value
    }

    public init(
        wallet: Wallet,
        service: any GemNotificationServiceProtocol,
        onOpenAction: ((UrlAction) -> Void)? = nil,
    ) {
        self.service = service
        self.onOpenAction = onOpenAction
        query = ObservableQuery(InAppNotificationsQuery(walletId: wallet.id.id), initialValue: [])
    }

    public var title: String {
        Localized.Settings.Notifications.title
    }

    public var rows: [GemNotificationRow] {
        notificationRows(notifications: notifications.map { $0.toGem() })
    }

    public func phase(_ rows: [GemNotificationRow]) -> GemListPhase {
        notificationListPhase(rows: rows, state: loadState)
    }

    public func sections(_ rows: [GemNotificationRow]) -> [ListSection<GemNotificationRow>] {
        DateSectionBuilder(items: rows, dateKeyPath: \.createdAt).build()
    }
}

// MARK: - Actions

public extension InAppNotificationsSceneViewModel {
    func load() async {
        loadState = await service.refresh()
    }

    func open(destination: GemNotificationDestination) {
        switch destination {
        case let .inApp(action): onOpenAction?(action)
        case let .web(url): url.asURL.map { UIApplication.shared.open($0) }
        }
    }
}
