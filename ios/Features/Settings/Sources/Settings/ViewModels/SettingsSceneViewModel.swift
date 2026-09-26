// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemListSection
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
public final class SettingsSceneViewModel {
    private let service: any GemSettingsServiceProtocol
    private let observablePreferences: ObservablePreferences

    public let walletsQuery = ObservableQuery(WalletsQuery(isPinned: .none), initialValue: [Wallet]())

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
}

public extension SettingsSceneViewModel {
    var sections: [GemListSection] {
        observablePreferences.changes
        return service.sections(
            wallets: walletsQuery.value.map { $0.toGem() },
            notificationsAvailable: true,
            walletConnectAvailable: true,
        )
    }
}
