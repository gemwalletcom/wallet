// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemSettingsServiceProtocol
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Store
import Style
import enum Gemstone.GemListRow
import PrimitivesComponents
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

    func destination(for row: GemListRow) -> SettingsRowDestination? {
        SettingsRowDestination(row: row)
    }
}

extension SettingsViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] {
        service.sections(
            wallets: walletsQuery.value.map { $0.toGem() },
            notificationsAvailable: true,
            walletConnectAvailable: true,
        ).listSections
    }
}
