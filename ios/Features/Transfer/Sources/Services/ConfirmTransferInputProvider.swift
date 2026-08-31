// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemTransferService
import struct Gemstone.GemConfirmData
import struct Gemstone.GemFeeAsset
import protocol Gemstone.GemConfirmServiceProtocol
import class Gemstone.GemAmountService
import class Gemstone.GemFeeService
import GemstonePrimitives
import Primitives
import Store
import PrimitivesComponents
import Validators

public struct ConfirmTransferInputProvider: Sendable {
    private let transferTransactionProvider: any TransferTransactionProvidable
    private let assetStore: AssetStore
    private let confirmService: any GemConfirmServiceProtocol

    private let feeService: GemFeeService
    private let transferService: GemTransferService
    private let amountService: GemAmountService

    public init(
        transferTransactionProvider: any TransferTransactionProvidable,
        assetStore: AssetStore,
        confirmService: any GemConfirmServiceProtocol,
        feeService: GemFeeService,
        transferService: GemTransferService,
        amountService: GemAmountService,
    ) {
        self.transferTransactionProvider = transferTransactionProvider
        self.assetStore = assetStore
        self.confirmService = confirmService
        self.feeService = feeService
        self.transferService = transferService
        self.amountService = amountService
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
        let feeAssetData = try assetStore.getAssetData(walletId: request.wallet.id, assetId: feeAssetId)
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
            transferAmount: TransferAmountCalculator(amountService: amountService).validate(
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

    func feeAssets(walletId: WalletId, chain: Chain) throws -> [GemFeeAsset] {
        try confirmService.feeAssets(walletId: walletId.id, chain: chain.rawValue)
    }

    private func preloadFailureError(metadata: TransferDataMetadata) -> TransferAmountCalculatorError? {
        guard feeService.isInsufficientNetworkFee(feeAssetId: metadata.feeAssetId.identifier, feeAvailable: metadata.feeAvailable.description) else {
            return nil
        }
        return .insufficientNetworkFee(metadata.feeAssetId.chain.asset, requirement: nil)
    }
}
