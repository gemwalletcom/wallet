// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum ReceiveAssetType: String, Hashable, Identifiable, Sendable {
    case asset
    case collection

    public var id: String {
        rawValue
    }
}
