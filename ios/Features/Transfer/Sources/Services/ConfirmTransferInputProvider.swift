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

    func load(
        request: ConfirmTransferRequest,
        metadata: TransferDataMetadata,
        selection: FeeSelection,
        feeAssetSelection: FeeAssetSelection,
    ) async throws -> ConfirmTransferPreload {
        let loadedTransactionData: TransferTransactionData
        do {
            loadedTransactionData = try await transferTransactionProvider.loadTransferTransactionData(
                wallet: request.wallet,
                data: request.data,
                selection: selection,
                available: metadata.available,
            )
        } catch {
            throw preloadFailureError(metadata: metadata) ?? error
        }

        let transactionData = switch feeAssetSelection {
        case .automatic: loadedTransactionData
        case let .selected(assetId): loadedTransactionData.withFeeAsset(assetId)
        }
        let fee = transactionData.transactionData.fee
        let feeAssetId = fee.feeAssetId
        let feeAssetData = try feeAssetProvider.getAssetData(walletId: request.wallet.id, assetId: feeAssetId)
        let assetPrices = if let feeAssetPrice = feeAssetData.price {
            metadata.assetPrices.merging([feeAssetId: feeAssetPrice]) { _, feeAssetPrice in feeAssetPrice }
        } else {
            metadata.assetPrices
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
    }

    func feeAssets(walletId: WalletId, chain: Chain) async throws -> [AssetData] {
        try await feeAssetProvider.feeAssets(walletId: walletId, chain: chain)
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
