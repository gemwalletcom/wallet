// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct StoredBalance: Sendable {
    public let assetId: AssetId
    public let balance: Balance
    public let isActive: Bool
}
