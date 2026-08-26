// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
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

    func map() throws -> TransactionInput {
        try TransactionInput(
            type: inputType.map(),
            asset: Primitives.Asset(inputType.getAsset()),
            senderAddress: senderAddress,
            destinationAddress: destinationAddress,
            value: BigInt.from(string: value),
            balance: BigInt.zero, // Would need to be provided from context
            gasPrice: gasPrice.map(),
            memo: memo,
            metadata: metadata,
        )
    }
}

public extension TransactionInput {
    func map() throws -> GemTransactionLoadInput {
        try GemTransactionLoadInput(
            inputType: type.map(),
            senderAddress: senderAddress,
            destinationAddress: destinationAddress,
            value: value.description,
            gasPrice: gasPrice.map(),
            memo: memo,
            isMaxValue: feeInput.isMaxAmount,
            metadata: metadata,
        )
    }
}
