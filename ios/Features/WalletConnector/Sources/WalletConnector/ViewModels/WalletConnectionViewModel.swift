// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

public struct WalletConnectionViewModel: Sendable {
    let connection: WalletConnection

    var nameText: String {
        connection.session.metadata.shortName
    }

    var imageUrl: URL? {
        connection.session.metadata.transactionAppMetadata.iconURL
    }

    var hostText: String? {
        url?.cleanHost()
    }

    var url: URL? {
        URL(string: connection.session.metadata.url)
    }
}
