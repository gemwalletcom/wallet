// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
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
