// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemTransferService
import struct Gemstone.GemConfirmData
import class Gemstone.GemFeeService
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Validators

public struct ConfirmTransferInputProvider: Sendable {
    private let transferTransactionProvider: any TransferTransactionProvidable
    private let feeAssetProvider: any FeeAssetProvidable

    private let feeService: GemFeeService
    private let transferService: GemTransferService

    public init(
        transferTransactionProvider: any TransferTransactionProvidable,
        feeAssetProvider: any FeeAssetProvidable,
        feeService: GemFeeService,
        transferService: GemTransferService,
    ) {
        self.transferTransactionProvider = transferTransactionProvider
        self.feeAssetProvider = feeAssetProvider
        self.feeService = feeService
        self.transferService = transferService
    }

    func load(
        request: ConfirmTransferRequest,
        metadata: TransferDataMetadata,
        selection: FeeSelection,
        feeAssetSelection: FeeAssetSelection,
    ) async throws -> ConfirmTransferPreload {
        let confirmData: GemConfirmData
        do {
            confirmData = try await transferTransactionProvider.loadConfirmData(
                wallet: request.wallet,
                data: request.data,
                selection: selection,
                feeAssetId: feeAssetSelection.selectedAssetId,
            )
        } catch {
            throw preloadFailureError(metadata: metadata) ?? error
        }
        let fee = try confirmData.fee.map()
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
            confirmData: confirmData,
            fee: fee,
            transferAmount: TransferAmountCalculator().validate(
                transferData: request.data,
                availableValue: try request.data.availableValue(balance: metadata.assetBalance, transferService: transferService),
                feeAsset: feeAssetData.asset,
                assetFeeBalance: metadata.feeAvailable,
                fee: fee.fee,
            ),
            feeAsset: feeAssetData.asset,
        )
        return try ConfirmTransferPreload(
            metadata: metadata,
            input: input,
            feeRates: confirmData.feeRates.map { try $0.map() },
            simulation: confirmData.simulation.map { try Primitives.SimulationResult($0) },
        )
    }

    func feeAssets(walletId: WalletId, chain: Chain) async throws -> [AssetData] {
        try await feeAssetProvider.feeAssets(walletId: walletId, chain: chain)
    }

    private func preloadFailureError(metadata: TransferDataMetadata) -> TransferAmountCalculatorError? {
        guard feeService.isInsufficientNetworkFee(feeAssetId: metadata.feeAssetId.identifier, feeAvailable: metadata.feeAvailable.description) else {
            return nil
        }
        return .insufficientNetworkFee(metadata.feeAssetId.chain.asset, requirement: nil)
    }
}
