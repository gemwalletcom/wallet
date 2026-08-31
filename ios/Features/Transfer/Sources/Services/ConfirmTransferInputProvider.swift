// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemTransferAmountResult
import enum Gemstone.GemTransferAmountError
import class Gemstone.GemTransferService
import struct Gemstone.GemConfirmData
import struct Gemstone.GemFeeAsset
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemConfirmLoadOptions
import enum Gemstone.GemConfirmError
import protocol Gemstone.GemConfirmServiceProtocol
import class Gemstone.GemAmountService
import class Gemstone.GemFeeService
import GemstonePrimitives
import Primitives
import Store
import PrimitivesComponents
import Validators

public struct ConfirmTransferInputProvider: Sendable {
    private let confirmService: any GemConfirmServiceProtocol

    private let feeService: GemFeeService
    private let transferService: GemTransferService
    private let amountService: GemAmountService

    public init(
        confirmService: any GemConfirmServiceProtocol,
        feeService: GemFeeService,
        transferService: GemTransferService,
        amountService: GemAmountService,
    ) {
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
        let account = try request.wallet.account(for: request.data.chain)
        let preload: GemConfirmPreload
        do {
            preload = try await confirmService.preload(
                walletId: request.wallet.id.id,
                input: request.data.confirmInput(from: account),
                options: GemConfirmLoadOptions(
                    feeSelection: selection.map(),
                    feeAssetId: feeAssetSelection.selectedAssetId?.identifier,
                ),
            )
        } catch let error as GemConfirmError {
            throw preloadFailureError(metadata: metadata) ?? error.map(symbol: request.data.type.asset.symbol)
        }
        let fee = try preload.confirmData.fee.map()
        let feeAsset = try Asset(preload.feeAsset)
        return try ConfirmTransferPreload(
            metadata: preload.metadata.map(assetId: metadata.assetId, feeAssetId: feeAsset.id),
            input: ConfirmTransferInput(
                confirmData: preload.confirmData,
                fee: fee,
                transferAmount: preload.amount.map(asset: request.data.type.asset, feeAsset: feeAsset),
                feeAsset: feeAsset,
            ),
            feeRates: preload.confirmData.feeRates.map { try $0.map() },
            simulation: preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
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

extension GemTransferAmountResult {
    func map(asset: Asset, feeAsset: Asset) -> TransferAmountValidation {
        switch self {
        case let .amount(amount):
            .success(TransferAmount(
                value: (try? BigInt.from(string: amount.value)) ?? .zero,
                networkFee: (try? BigInt.from(string: amount.networkFee)) ?? .zero,
                useMaxAmount: amount.isMaxAmount,
            ))
        case let .error(error):
            .failure(TransferAmountCalculatorError(mapped(error), asset: asset, assetFee: feeAsset))
        }
    }

    private func mapped(_ error: GemTransferAmountError) -> TransferAmountError {
        (try? error.map()) ?? .insufficientBalance(
            assetId: asset(from: error),
            requirement: BalanceRequirement(required: .zero, available: .zero),
        )
    }

    private func asset(from error: GemTransferAmountError) -> AssetId {
        switch error {
        case let .InsufficientBalance(assetId, _, _),
             let .InsufficientNetworkFee(assetId, _, _),
             let .MinimumAccountBalanceTooLow(assetId, _, _):
            (try? AssetId(id: assetId)) ?? .init(chain: .bitcoin, tokenId: nil)
        }
    }
}

private extension GemConfirmError {
    func map(symbol: String) -> Error {
        switch self {
        case .ScanMalicious: ScanTransactionError.malicious
        case .ScanMemoRequired: ScanTransactionError.memoRequired(symbol: symbol)
        case .FeeRatesMissing: ChainCoreError.feeRateMissed
        case .Load, .Broadcast, .Network, .Offline, .Record, .AccountMissing, .BalanceMissing, .SenderMismatch, .Sign, .ApprovalInvalid: self
        }
    }
}
