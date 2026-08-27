// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import BigInt
import Gemstone
import GemstonePrimitives
import Primitives

public protocol TransferTransactionProvidable: Sendable {
    func loadTransferTransactionData(
        wallet: Primitives.Wallet,
        data: TransferData,
        selection: FeeSelection,
        feeAssetId: Primitives.AssetId?,
    ) async throws -> TransferTransactionData
}

public struct TransferTransactionProvider: TransferTransactionProvidable {
    private let confirmService: GemConfirmService

    public init(confirmService: GemConfirmService) {
        self.confirmService = confirmService
    }

    public func loadTransferTransactionData(
        wallet: Primitives.Wallet,
        data: TransferData,
        selection: FeeSelection,
        feeAssetId: Primitives.AssetId?,
    ) async throws -> TransferTransactionData {
        let account = try wallet.account(for: data.chain)
        let result: GemConfirmData
        do {
            result = try await confirmService.load(
                input: data.confirmInput(from: account),
                options: GemConfirmLoadOptions(feeSelection: selection.map(), feeAssetId: feeAssetId?.identifier),
            )
        } catch let error as GemConfirmError {
            throw error.map(symbol: data.type.asset.symbol)
        }
        return try TransferTransactionData(
            allRates: result.feeRates.map { try $0.map() },
            transactionData: TransactionData(
                fee: result.fee.map(),
                metadata: result.metadata,
            ),
            scanResult: result.scan.map { try Primitives.ScanTransaction($0) },
            simulation: result.simulation.map { try Primitives.SimulationResult($0) },
        )
    }
}

private extension GemConfirmError {
    func map(symbol: String) -> Error {
        switch self {
        case .ScanMalicious: ScanTransactionError.malicious
        case .ScanMemoRequired: ScanTransactionError.memoRequired(symbol: symbol)
        case .FeeRatesMissing: ChainCoreError.feeRateMissed
        case .Load, .Broadcast, .Network, .Offline: self
        }
    }
}
