// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemSettingsServiceProtocol
import enum Gemstone.GemSettingsRow
import struct Gemstone.GemSettingsSection
import struct Gemstone.Rewards
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Store
import Style
import SwiftUI
import GemstoneServices

@Observable
@MainActor
public final class SettingsViewModel {
    private let service: any GemSettingsServiceProtocol
    private let observablePreferences: ObservablePreferences

    public let walletsQuery = ObservableQuery(WalletsRequest(isPinned: .none), initialValue: [Wallet]())

    public init(
        service: any GemSettingsServiceProtocol,
        observablePreferences: ObservablePreferences,
    ) {
        self.service = service
        self.observablePreferences = observablePreferences
    }

    var title: String {
        Localized.Settings.title
    }

    var sections: [GemSettingsSection] {
        service.sections(
            wallets: walletsQuery.value.map { $0.toGem() },
            notificationsAvailable: true,
            walletConnectAvailable: true,
        )
    }

    func listItem(for row: GemSettingsRow) -> ListItemModel {
        ListItemModel(
            title: row.title,
            subtitle: subtitle(for: row),
            imageStyle: .settings(assetImage: row.assetImage),
        )
    }

    private var walletsValue: String {
        "\(walletsQuery.value.count)"
    }

    private func subtitle(for row: GemSettingsRow) -> String? {
        switch row {
        case .wallets: walletsValue
        case .security, .notifications, .preferences, .walletConnect, .support, .rewards, .aboutUs, .developer: nil
        }
    }

}
