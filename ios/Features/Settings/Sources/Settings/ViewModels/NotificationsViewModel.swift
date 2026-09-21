// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemNotificationsServiceProtocol
import enum Gemstone.GemPushResult
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class NotificationsViewModel {
    private let service: any GemNotificationsServiceProtocol

    var isEnabled: Bool
    var isPresentingAlertMessage: AlertMessage?

    public init(service: any GemNotificationsServiceProtocol) {
        self.service = service
        isEnabled = service.isEnabled()
    }

    var title: String {
        Localized.Settings.Notifications.title
    }

    var priceAlertsListItem: ListItemModel {
        ListItemModel(
            title: Localized.Settings.PriceAlerts.title,
            imageStyle: .settings(assetImage: AssetImage.image(Images.Settings.priceAlerts)),
        )
    }
}

// MARK: - Business Logic

extension NotificationsViewModel {
    func enable(isEnabled: Bool) async {
        let state = await service.setEnabled(enabled: isEnabled)
        self.isEnabled = state.isEnabled
        if case let .notRegistered(error) = state.result {
            isPresentingAlertMessage = AlertMessage(message: error.text)
        }
    }
}
