// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftHTTPClient

public enum GemAPI: TargetType {
    case getAssets([AssetId], currency: String?)

    public var baseUrl: URL {
        Constants.apiURL
    }

    public var method: HTTPMethod {
        .POST
    }

    public var path: String {
        switch self {
        case let .getAssets(_, currency):
            var path = "/v1/assets"
            if let currency {
                path += "?currency=\(currency)"
            }
            return path
        }
    }

    public var data: RequestData {
        switch self {
        case let .getAssets(value, _):
            .encodable(value.map(\.identifier))
        }
    }
}
