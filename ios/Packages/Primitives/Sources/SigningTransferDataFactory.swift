// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public enum SigningTransferDataFactory {
    public static func transferData(
        asset: Asset,
        appMetadata: TransactionAppMetadata,
        transaction: SignableTransaction,
        outputAction: TransferDataOutputAction,
        payment: PaymentData? = .none,
    ) throws -> TransferData {
        switch transaction {
        case let .ethereum(transaction, transactionType):
            try ethereumTransferData(asset: asset, appMetadata: appMetadata, transaction: transaction, transactionType: transactionType, payment: payment)
        case let .solana(transaction, outputType),
             let .sui(transaction, outputType),
             let .ton(transaction, outputType),
             let .tron(transaction, outputType):
            encodedTransferData(asset: asset, appMetadata: appMetadata, transaction: transaction, outputType: outputType, outputAction: outputAction, payment: payment)
        }
    }

    public static func ethereumTransferData(
        asset: Asset,
        appMetadata: TransactionAppMetadata,
        transaction: EthereumTransactionData,
        transactionType: TransactionType,
        payment: PaymentData? = .none,
    ) throws -> TransferData {
        let address = transaction.to
        let value = try BigInt.fromHex(transaction.value ?? .zero)
        let gasLimit: BigInt? = {
            if let value = transaction.gasLimit {
                return BigInt(hex: value)
            } else if let gas = transaction.gas {
                return BigInt(hex: gas)
            }
            return .none
        }()

        let gasPrice: GasPriceType? = {
            if let maxFeePerGas = transaction.maxFeePerGas,
               let maxPriorityFeePerGas = transaction.maxPriorityFeePerGas,
               let maxFeePerGasBigInt = BigInt(hex: maxFeePerGas),
               let maxPriorityFeePerGasBigInt = BigInt(hex: maxPriorityFeePerGas)
            {
                return .eip1559(gasPrice: maxFeePerGasBigInt, priorityFee: maxPriorityFeePerGasBigInt)
            }
            return .none
        }()

        let data: Data? = {
            if let data = transaction.data {
                return Data(fromHex: data)
            }
            return .none
        }()

        return TransferData(
            type: Self.type(asset: asset, appMetadata: appMetadata, payment: payment, extra: TransferDataExtra(
                to: address,
                gasLimit: gasLimit,
                gasPrice: gasPrice,
                data: data,
                transactionType: transactionType,
            )),
            recipientData: RecipientData(
                recipient: Recipient(name: .none, address: address, memo: .none),
                amount: .none,
            ),
            amount: .exact(value),
        )
    }

    public static func encodedTransferData(
        asset: Asset,
        appMetadata: TransactionAppMetadata,
        transaction: String,
        outputType: TransferDataOutputType,
        outputAction: TransferDataOutputAction,
        payment: PaymentData? = .none,
    ) -> TransferData {
        TransferData(
            type: type(
                asset: asset,
                appMetadata: appMetadata,
                payment: payment,
                extra: TransferDataExtra(
                    to: "",
                    data: transaction.data(using: .utf8),
                    outputType: outputType,
                    outputAction: outputAction,
                ),
            ),
            recipientData: RecipientData(
                recipient: Recipient(name: .none, address: "", memo: .none),
                amount: .none,
            ),
            amount: .exact(.zero),
        )
    }
}

// MARK: - Private

private extension SigningTransferDataFactory {
    static func type(asset: Asset, appMetadata: TransactionAppMetadata, payment: PaymentData?, extra: TransferDataExtra) -> TransferDataType {
        guard let payment else {
            return .generic(asset: asset, appMetadata: appMetadata, extra: extra)
        }
        return .payment(asset: asset, payment: payment, extra: extra)
    }
}
