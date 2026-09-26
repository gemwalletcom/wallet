// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemLoadState
import enum Gemstone.GemNotificationDestination
import protocol Gemstone.GemNotificationServiceProtocol
import func Gemstone.loadError
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
    private let wallet: Wallet
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
        self.wallet = wallet
        self.service = service
        self.onOpenAction = onOpenAction
        query = ObservableQuery(InAppNotificationsQuery(walletId: wallet.id.id), initialValue: [])
    }

    public var title: String {
        Localized.Settings.Notifications.title
    }

    public var loadError: Error? {
        Gemstone.loadError(state: loadState, hasRows: !notifications.isEmpty)
    }

    public var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .notifications)
    }

    public var sections: [ListSection<InAppNotificationListItemViewModel>] {
        let notifications = notifications
        let items = zip(notifications, notificationRows(notifications: notifications.map { $0.toGem() })).map {
            InAppNotificationListItemViewModel(notification: $0, row: $1)
        }
        return DateSectionBuilder(items: items, dateKeyPath: \.createdAt).build()
    }
}

// MARK: - Actions

public extension InAppNotificationsSceneViewModel {
    func load() async {
        loadState = await service.refresh(hasNotifications: notifications.isNotEmpty)
    }

    func open(destination: GemNotificationDestination) {
        switch destination {
        case let .inApp(action): onOpenAction?(action)
        case let .web(url): url.asURL.map { UIApplication.shared.open($0) }
        }
    }
}
