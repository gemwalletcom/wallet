// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemNftSearchItem
import Primitives

public extension GemNftSearchItem {
    func map() throws -> NFTSearchItem {
        switch self {
        case let .collection(data): try .collection(NFTData(data))
        case let .asset(data): try .asset(NFTAssetData(data))
        }
    }
}
