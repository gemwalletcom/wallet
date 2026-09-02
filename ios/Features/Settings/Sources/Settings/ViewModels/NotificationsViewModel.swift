// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNotificationsServiceProtocol
import Components
import Foundation
import Localization
import Primitives
import Style

@Observable
@MainActor
public final class NotificationsViewModel {
    private let service: any GemNotificationsServiceProtocol

    var isEnabled: Bool

    public init(service: any GemNotificationsServiceProtocol) {
        self.service = service
        isEnabled = service.isEnabled()
    }

    var title: String {
        Localized.Settings.Notifications.title
    }

    var priceAlertsTitle: String {
        Localized.Settings.PriceAlerts.title
    }

    var priceAlertsImage: AssetImage {
        AssetImage.image(Images.Settings.priceAlerts)
    }
}

// MARK: - Business Logic

extension NotificationsViewModel {
    func enable(isEnabled: Bool) async throws {
        self.isEnabled = try await service.setEnabled(enabled: isEnabled)
    }
}
