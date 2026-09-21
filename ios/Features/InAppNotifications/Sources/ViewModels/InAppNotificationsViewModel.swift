// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemNotificationDestination
import protocol Gemstone.GemNotificationServiceProtocol
import enum Gemstone.UrlAction
import Localization
import Primitives
import PrimitivesComponents
import Store
import UIKit

@Observable
@MainActor
public final class InAppNotificationsViewModel {
    private let service: any GemNotificationServiceProtocol
    private let wallet: Wallet
    private let onOpenAction: ((UrlAction) -> Void)?

    public let query: ObservableQuery<InAppNotificationsRequest>
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
        query = ObservableQuery(InAppNotificationsRequest(walletId: wallet.id.id), initialValue: [])
    }

    public var title: String {
        Localized.Settings.Notifications.title
    }

    public var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .notifications)
    }

    public var sections: [ListSection<InAppNotificationListItemViewModel>] {
        DateSectionBuilder(
            items: notifications,
            dateKeyPath: \.createdAt,
            transform: { InAppNotificationListItemViewModel(notification: $0) },
        ).build()
    }
}

// MARK: - Actions

public extension InAppNotificationsViewModel {
    func load() async {
        do {
            try await service.open()
        } catch {
            debugLog("load notifications error: \(error)")
        }
    }

    func open(destination: GemNotificationDestination) {
        switch destination {
        case let .inApp(action): onOpenAction?(action)
        case let .web(url): url.asURL.map { UIApplication.shared.open($0) }
        }
    }
}
