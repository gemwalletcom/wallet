// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemConnectionDetails
import enum Gemstone.GemConnectionDetailRow
import Localization

public struct ConnectionSceneViewModel: Sendable {
    private static let dateFormatter = RelativeDateFormatter()

    let details: GemConnectionDetails

    init(details: GemConnectionDetails) {
        self.details = details
    }

    var title: String {
        Localized.WalletConnect.Connection.title
    }

    var disconnectTitle: String {
        Localized.WalletConnect.disconnect
    }

    func listItem(for row: GemConnectionDetailRow) -> ListItemModel {
        ListItemModel(title: row.title, subtitle: value(for: row))
    }

    func value(for row: GemConnectionDetailRow) -> String {
        switch row {
        case .wallet: details.wallet
        case .date: Self.dateFormatter.string(from: details.date)
        }
    }
}
