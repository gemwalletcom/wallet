// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransactionLoadFee {
    func map() throws -> Fee {
        Fee(
            fee: fee,
            gasPriceType: gasPriceType.map(),
            gasLimit: gasLimit,
            options: options.map(),
            feeAssetId: try AssetId(id: feeAsset),
        )
    }
}

public extension Fee {
    func map() -> Gemstone.GemTransactionLoadFee {
        Gemstone.GemTransactionLoadFee(
            fee: fee,
            gasPriceType: gasPriceType.map(),
            gasLimit: gasLimit,
            options: options.map(),
            feeAsset: feeAssetId.identifier,
        )
    }
}
