// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import Transfer

public struct TransferTransactionProviderMock: TransferTransactionProvidable {
    public var result: Result<TransferTransactionData, Error>

    public init(result: Result<TransferTransactionData, Error>) {
        self.result = result
    }

    public func loadTransferTransactionData(
        wallet _: Wallet,
        data _: TransferData,
        priority _: FeePriority,
        available _: BigInt,
    ) async throws -> TransferTransactionData {
        try result.get()
    }
}
