// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemListSection
import protocol Gemstone.GemNotificationsServiceProtocol
import enum Gemstone.GemPushResult
import enum Gemstone.GemRowTap
import func Gemstone.notificationsSections
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
}

public extension NotificationsViewModel {
    var sections: [GemListSection] {
        notificationsSections(pushEnabled: isEnabled)
    }
}

// MARK: - Business Logic

extension NotificationsViewModel {
    func onToggle(_ tap: GemRowTap, _ isOn: Bool) {
        switch tap {
        case .pushNotifications: isEnabled = isOn
        default: break
        }
    }

    func enable(isEnabled: Bool) async {
        let state = await service.setEnabled(enabled: isEnabled)
        self.isEnabled = state.isEnabled
        if case let .notRegistered(error) = state.result {
            isPresentingAlertMessage = AlertMessage(message: error.text)
        }
    }
}
