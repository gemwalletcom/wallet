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
            return ConfirmTransferInput(
                transactionData: transactionData.transactionData,
                feeRates: transactionData.rates,
                transferAmount: TransferAmountCalculator().validate(
                    transferData: request.data,
                    availableValue: request.data.availableValue(metadata: metadata),
                    assetFeeBalance: metadata.assetFeeBalance.available,
                    fee: transactionData.transactionData.fee.fee,
                ),
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
