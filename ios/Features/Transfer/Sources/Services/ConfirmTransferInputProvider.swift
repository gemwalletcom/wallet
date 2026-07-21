// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import Validators

public struct ConfirmTransferInputProvider: Sendable {
    private let transferTransactionProvider: any TransferTransactionProvidable

    public init(transferTransactionProvider: any TransferTransactionProvidable) {
        self.transferTransactionProvider = transferTransactionProvider
    }

    public func load(
        request: ConfirmTransferRequest,
        metadata: TransferDataMetadata,
        selection: FeeSelection,
    ) async throws -> ConfirmTransferInput {
        do {
            let transactionData = try await transferTransactionProvider.loadTransferTransactionData(
                wallet: request.wallet,
                data: request.data,
                selection: selection,
                available: metadata.available,
            )
            let transferAmountInput = TransferAmountInput(
                asset: request.data.type.asset,
                assetBalance: metadata.assetBalance,
                value: request.data.value,
                availableValue: request.data.availableValue(metadata: metadata),
                assetFee: request.data.type.asset.feeAsset,
                assetFeeBalance: metadata.assetFeeBalance,
                fee: transactionData.transactionData.fee.fee,
                transferData: request.data,
            )
            return ConfirmTransferInput(
                transactionData: transactionData.transactionData,
                feeRates: transactionData.rates,
                transferAmount: TransferAmountCalculator().validate(input: transferAmountInput),
            )
        } catch {
            throw insufficientNetworkFeeError(metadata: metadata) ?? error
        }
    }

    private func insufficientNetworkFeeError(metadata: TransferDataMetadata) -> TransferAmountCalculatorError? {
        do {
            try TransferAmountCalculator().validateNetworkFee(metadata.feeAvailable, feeAssetId: metadata.feeAssetId)
            return nil
        } catch {
            return error
        }
    }
}
