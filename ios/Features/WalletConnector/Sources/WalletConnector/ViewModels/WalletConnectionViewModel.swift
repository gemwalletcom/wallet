// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApplicationMetadataService
import struct Gemstone.GemConnectionRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public struct WalletConnectionViewModel: Sendable {
    let connection: WalletConnection

    private var row: GemConnectionRow {
        GemApplicationMetadataService.shared.connectionRow(metadata: connection.session.metadata.map())
    }

    var nameText: String {
        row.title
    }

    var imageUrl: URL? {
        connection.session.metadata.iconURL
    }

    var hostText: String? {
        row.host
    }

    var url: URL? {
        URL(string: connection.session.metadata.url)
    }
}
