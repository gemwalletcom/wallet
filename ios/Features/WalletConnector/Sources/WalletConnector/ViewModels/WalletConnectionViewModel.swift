// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public struct WalletConnectionViewModel: Sendable {
    let connection: WalletConnection

    var nameText: String {
        connection.session.metadata.shortName
    }

    var imageUrl: URL? {
        connection.session.metadata.iconURL
    }

    var hostText: String? {
        let host = connection.session.metadata.host
        return host.isEmpty ? nil : host
    }

    var url: URL? {
        URL(string: connection.session.metadata.url)
    }
}
