// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import BigInt
import Gemstone
import GemstonePrimitives
import Primitives

public protocol TransferTransactionProvidable: Sendable {
    func loadConfirmData(
        wallet: Primitives.Wallet,
        data: TransferData,
        selection: FeeSelection,
        feeAssetId: Primitives.AssetId?,
    ) async throws -> GemConfirmData
}

public struct TransferTransactionProvider: TransferTransactionProvidable {
    private let confirmService: any GemConfirmServiceProtocol

    public init(confirmService: any GemConfirmServiceProtocol) {
        self.confirmService = confirmService
    }

    public func loadConfirmData(
        wallet: Primitives.Wallet,
        data: TransferData,
        selection: FeeSelection,
        feeAssetId: Primitives.AssetId?,
    ) async throws -> GemConfirmData {
        let account = try wallet.account(for: data.chain)
        do {
            return try await confirmService.load(
                input: data.confirmInput(from: account),
                options: GemConfirmLoadOptions(feeSelection: selection.map(), feeAssetId: feeAssetId?.identifier),
            )
        } catch let error as GemConfirmError {
            throw error.map(symbol: data.type.asset.symbol)
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
