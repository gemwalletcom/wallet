// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemTransactionLoadInput {
    static func map(signerInput: SignerInput) throws -> GemTransactionLoadInput {
        try GemTransactionLoadInput(
            inputType: signerInput.type.map(),
            senderAddress: signerInput.senderAddress,
            destinationAddress: signerInput.destinationAddress,
            value: signerInput.value.description,
            gasPrice: signerInput.fee.gasPriceType.map(),
            memo: signerInput.memo,
            isMaxValue: signerInput.useMaxAmount,
            metadata: signerInput.metadata,
        )
    }
}
