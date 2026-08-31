// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApplicationMetadataService
import Primitives
import PrimitivesComponents

public struct WalletConnectionViewModel: Sendable {
    let connection: WalletConnection
    let applicationMetadataService: GemApplicationMetadataService

    var nameText: String {
        connection.session.metadata.shortName(applicationMetadataService: applicationMetadataService)
    }

    var imageUrl: URL? {
        if let url = URL(string: connection.session.metadata.icon) {
            if url.host() == nil {
                return URL(string: connection.session.metadata.url + connection.session.metadata.icon)
            }
            return url
        }
        return .none
    }

    var hostText: String? {
        url?.cleanHost()
    }

    var url: URL? {
        URL(string: connection.session.metadata.url)
    }
}
