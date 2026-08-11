// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct ConfirmTransferPreload: Sendable {
    public let metadata: TransferDataMetadata
    public let input: ConfirmTransferInput
    public let feeRates: [FeeRate]

    public init(
        metadata: TransferDataMetadata,
        input: ConfirmTransferInput,
        feeRates: [FeeRate],
    ) {
        self.metadata = metadata
        self.input = input
        self.feeRates = feeRates
    }
}
