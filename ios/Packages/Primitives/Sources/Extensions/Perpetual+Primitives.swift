// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension PerpetualMarginType {
    init(id: String) throws {
        if let type = PerpetualMarginType(rawValue: id) {
            self = type
        } else {
            throw AnyError("invalid margin type: \(id)")
        }
    }
}

public extension PerpetualDirection {
    init(id: String) throws {
        if let direction = PerpetualDirection(rawValue: id) {
            self = direction
        } else {
            throw AnyError("invalid direction: \(id)")
        }
    }
}

public extension Perpetual {
    var coin: String {
        assetId.tokenId?.components(separatedBy: AssetId.subTokenSeparator).last ?? name
    }

    var marginType: PerpetualMarginType {
        isIsolatedOnly ? .isolated : .cross
    }
}

public extension PerpetualSearchData {
    var assetBasic: AssetBasic {
        AssetBasic(
            asset: asset,
            properties: AssetProperties(
                isEnabled: false,
                isBuyable: false,
                isSellable: false,
                isSwapable: false,
                isStakeable: false,
                stakingApr: nil,
                isEarnable: false,
                earnApr: nil,
                hasImage: false,
            ),
            score: AssetScore(rank: 0),
            price: nil,
        )
    }
}
