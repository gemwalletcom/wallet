// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import Transfer

public final class TransferTransactionProviderMock: TransferTransactionProvidable, @unchecked Sendable {
    public var result: Result<TransferTransactionData, Error>
    public private(set) var loadedFeeAssetId: AssetId?

    public init(result: Result<TransferTransactionData, Error>) {
        self.result = result
    }

    public func loadTransferTransactionData(
        wallet _: Wallet,
        data _: TransferData,
        selection _: FeeSelection,
        feeAssetId: AssetId?,
    ) async throws -> TransferTransactionData {
        loadedFeeAssetId = feeAssetId
        return try result.get()
    }
}
