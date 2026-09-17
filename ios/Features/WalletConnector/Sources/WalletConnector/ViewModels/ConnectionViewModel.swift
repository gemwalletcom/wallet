// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConnection

struct ConnectionViewModel {
    let connection: GemConnection

    var iconUrl: URL? {
        connection.row.iconUrl.flatMap(URL.init(string:))
    }

    var title: String {
        connection.row.title
    }

    var host: String? {
        connection.row.host
    }

    var websiteUrl: URL? {
        URL(string: connection.connection.session.metadata.url)
    }
}
