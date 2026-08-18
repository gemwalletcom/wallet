// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import Validators

public struct ConfirmTransferInputProvider: Sendable {
    private let transferTransactionProvider: any TransferTransactionProvidable
    private let feeAssetProvider: any FeeAssetProvidable

    public init(
        transferTransactionProvider: any TransferTransactionProvidable,
        feeAssetProvider: any FeeAssetProvidable,
    ) {
        self.transferTransactionProvider = transferTransactionProvider
        self.feeAssetProvider = feeAssetProvider
    }

    public func load(
        request: ConfirmTransferRequest,
        metadata: TransferDataMetadata,
        selection: FeeSelection,
    ) async throws -> ConfirmTransferPreload {
        do {
            let transactionData = try await transferTransactionProvider.loadTransferTransactionData(
                wallet: request.wallet,
                data: request.data,
                selection: selection,
                available: metadata.available,
            )
            let fee = transactionData.transactionData.fee
            let feeAssetId = fee.feeAssetId
            let feeAssetData = try feeAssetProvider.load(walletId: request.wallet.id, feeAssetId: feeAssetId)
            var assetPrices = metadata.assetPrices
            if let feePrice = feeAssetData.price {
                assetPrices[feeAssetId] = feePrice
            }
            let metadata = TransferDataMetadata(
                assetId: metadata.assetId,
                feeAssetId: feeAssetId,
                assetBalance: metadata.assetBalance,
                assetFeeBalance: feeAssetData.balance,
                assetPrices: assetPrices,
            )
            let input = ConfirmTransferInput(
                transactionData: transactionData.transactionData,
                transferAmount: TransferAmountCalculator().validate(
                    transferData: request.data,
                    availableValue: request.data.availableValue(metadata: metadata),
                    feeAsset: feeAssetData.asset,
                    assetFeeBalance: metadata.feeAvailable,
                    fee: fee.fee,
                ),
                feeAsset: feeAssetData.asset,
            )
            return ConfirmTransferPreload(
                metadata: metadata,
                input: input,
                feeRates: transactionData.rates,
            )
        } catch {
            throw preloadFailureError(metadata: metadata) ?? error
        }
    }

    private func preloadFailureError(metadata: TransferDataMetadata) -> TransferAmountCalculatorError? {
        if [Chain.hyperCore, Chain.tron].contains(metadata.feeAssetId.chain) {
            return nil
        }
        guard metadata.feeAvailable.isZero, metadata.feeAssetId.type == .native else {
            return nil
        }
        return .insufficientNetworkFee(metadata.feeAssetId.chain.asset, requirement: nil)
    }
}
