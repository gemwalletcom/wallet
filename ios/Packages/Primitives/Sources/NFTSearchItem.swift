// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum NFTSearchItem: Equatable, Sendable {
    case collection(NFTData)
    case asset(NFTAssetData)
}
