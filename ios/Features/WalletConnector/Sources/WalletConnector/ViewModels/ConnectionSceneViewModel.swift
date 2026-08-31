// Copyright (c). Gem Wallet. All rights reserved.

import WalletConnectorService
import Formatters
import Foundation
import Localization
import Primitives

public struct ConnectionSceneViewModel: Sendable {
    private static let dateFormatter = RelativeDateFormatter()

    let model: WalletConnectionViewModel
    let connector: any WalletConnectorServiceable

    var title: String {
        Localized.WalletConnect.Connection.title
    }

    var disconnectTitle: String {
        Localized.WalletConnect.disconnect
    }

    var walletField: String {
        Localized.Common.wallet
    }

    var walletText: String {
        model.connection.wallet.name
    }

    var dateField: String {
        Localized.Transaction.date
    }

    var dateText: String {
        Self.dateFormatter.string(from: model.connection.session.createdAt)
    }

    func disconnect() async throws {
        try await connector.disconnect(sessionId: model.connection.session.sessionId)
    }
}
