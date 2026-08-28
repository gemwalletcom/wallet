// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemConfirmData
import GemstonePrimitives
import Primitives
import Transfer

public final class TransferTransactionProviderMock: TransferTransactionProvidable, @unchecked Sendable {
    public var result: Result<GemConfirmData, Error>
    public private(set) var loadedFeeAssetId: AssetId?

    public init(result: Result<GemConfirmData, Error>) {
        self.result = result
    }

    public func loadConfirmData(
        wallet _: Wallet,
        data _: TransferData,
        selection _: FeeSelection,
        feeAssetId: AssetId?,
    ) async throws -> GemConfirmData {
        loadedFeeAssetId = feeAssetId
        return try result.get()
    }
}
