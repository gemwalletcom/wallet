// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum DeepLink: Equatable, Sendable {
    case asset(AssetId)
    case perpetuals
    case rewards(code: String?)
}
