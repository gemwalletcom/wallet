// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension TransactionData {
    static func mock(feeAsset: Asset = .mock()) -> TransactionData {
        TransactionData(fee: Fee(fee: 1, gasPriceType: .regular(gasPrice: 1), gasLimit: 1, feeAssetId: feeAsset.id))
    }
}
