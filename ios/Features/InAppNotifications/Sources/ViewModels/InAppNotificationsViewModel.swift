// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemNotificationServiceProtocol
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import UIKit

@Observable
@MainActor
public final class InAppNotificationsViewModel {
    private let service: any GemNotificationServiceProtocol
    private let wallet: Wallet

    public let query: ObservableQuery<InAppNotificationsRequest>
    public var notifications: [Primitives.InAppNotification] {
        query.value
    }

    public init(
        wallet: Wallet,
        service: any GemNotificationServiceProtocol,
    ) {
        self.wallet = wallet
        self.service = service
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
            try await service.open(walletId: wallet.id.id)
        } catch {
            debugLog("load notifications error: \(error)")
        }
    }

    func open(url: URL) {
        UIApplication.shared.open(url)
    }
}
