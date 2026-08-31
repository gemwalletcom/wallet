// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

struct SwapMetadataViewModel {
    let metadata: TransactionExtendedMetadata

    var headerInput: SwapHeaderInput? {
        guard
            let swapMetadata = metadata.swapMetadata,
            let fromAsset = metadata.asset(for: swapMetadata.fromAsset),
            let toAsset = metadata.asset(for: swapMetadata.toAsset),
            let fromValue = try? BigInt.from(string: swapMetadata.fromValue),
            let toValue = try? BigInt.from(string: swapMetadata.toValue)
        else {
            return .none
        }

        return SwapHeaderInput(
            from: AssetValuePrice(
                asset: fromAsset,
                value: fromValue,
                price: metadata.price(for: swapMetadata.fromAsset),
            ),
            to: AssetValuePrice(
                asset: toAsset,
                value: toValue,
                price: metadata.price(for: swapMetadata.toAsset),
            ),
        )
    }
}
