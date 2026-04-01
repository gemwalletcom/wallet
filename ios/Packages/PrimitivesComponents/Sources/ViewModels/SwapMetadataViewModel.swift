// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

struct SwapMetadataViewModel: Sendable {
    let metadata: TransactionExtendedMetadata

    init(metadata: TransactionExtendedMetadata) {
        self.metadata = metadata
    }

    var headerInput: SwapHeaderInput? {
        guard
            let swapMetadata = metadata.swapMetadata,
            let fromAsset = metadata.asset(for: swapMetadata.fromAsset),
            let toAsset = metadata.asset(for: swapMetadata.toAsset)
        else {
            return .none
        }

        return SwapHeaderInput(
            from: AssetValuePrice(
                asset: fromAsset,
                value: BigInt.fromString(swapMetadata.fromValue),
                price: metadata.price(for: swapMetadata.fromAsset),
            ),
            to: AssetValuePrice(
                asset: toAsset,
                value: BigInt.fromString(swapMetadata.toValue),
                price: metadata.price(for: swapMetadata.toAsset),
            ),
        )
    }
}
