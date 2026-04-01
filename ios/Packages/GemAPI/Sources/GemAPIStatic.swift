// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftHTTPClient

public enum GemAPIStatic: TargetType {
    case getValidators(chain: String)

    public var baseUrl: URL {
        URL(string: "https://assets.gemwallet.com")!
    }

    public var method: HTTPMethod {
        .GET
    }

    public var path: String {
        switch self {
        case let .getValidators(chain):
            "/blockchains/\(chain)/validators.json"
        }
    }

    public var data: RequestData {
        switch self {
        case .getValidators:
            .plain
        }
    }
}
