// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftHTTPClient

struct WidgetAssetsTarget: TargetType {
    let assetIds: [String]
    let currency: String

    var baseUrl: URL {
        URL(string: "https://api.gemwallet.com")!
    }

    var method: HTTPMethod {
        .POST
    }

    var path: String {
        "/v1/assets?currency=\(currency)"
    }

    var data: RequestData {
        .encodable(assetIds)
    }
}
