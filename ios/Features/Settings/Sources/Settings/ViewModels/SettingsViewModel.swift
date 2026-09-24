// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import protocol Gemstone.GemNotificationsServiceProtocol
import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class SettingsViewModel {
    private let service: any GemSettingsServiceProtocol
    private let notifications: any GemNotificationsServiceProtocol
    private let observablePreferences: ObservablePreferences

    public let walletsQuery = ObservableQuery(WalletsRequest(isPinned: .none), initialValue: [Wallet]())

    public init(
        service: any GemSettingsServiceProtocol,
        notifications: any GemNotificationsServiceProtocol,
        observablePreferences: ObservablePreferences,
    ) {
        self.service = service
        self.notifications = notifications
        self.observablePreferences = observablePreferences
    }

    /// Core decides whether opening support is a moment to ask for notifications.
    public func openSupport() async {
        _ = await notifications.enableForSupport()
    }

    var title: String {
        Localized.Settings.title
    }

    func destination(for row: GemListRow) -> SettingsRowDestination? {
        SettingsRowDestination(row: row)
    }
}

extension SettingsViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] {
        observablePreferences.changes
        return service.sections(
            wallets: walletsQuery.value.map { $0.toGem() },
            notificationsAvailable: true,
            walletConnectAvailable: true,
        ).listSections
    }
}
