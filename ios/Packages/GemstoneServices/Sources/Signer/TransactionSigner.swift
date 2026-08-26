// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
import GemstonePrimitives
import Primitives

public struct TransactionSigner: TransactionSigning {
    private let keystore: any Keystore

    public init(keystore: any Keystore) {
        self.keystore = keystore
    }

    public func sign(
        transfer: TransferData,
        transactionData: TransactionData,
        amount: TransferAmount,
        wallet: Wallet,
    ) async throws -> [GemSignedTransaction] {
        let fee = Fee(
            fee: amount.networkFee,
            gasPriceType: transactionData.fee.gasPriceType,
            gasLimit: transactionData.fee.gasLimit,
            options: transactionData.fee.options,
            feeAssetId: transactionData.fee.feeAssetId,
        )

        let input = try SignerInput(
            type: transfer.type,
            asset: transfer.type.asset,
            value: amount.value,
            fee: fee,
            isMaxAmount: amount.useMaxAmount,
            memo: transfer.recipientData.recipient.memo,
            senderAddress: wallet.account(for: transfer.type.chain).address,
            destinationAddress: transfer.recipientData.recipient.address,
            metadata: transactionData.metadata,
        )

        return try await keystore.sign(wallet: wallet, input: input)
    }
}
