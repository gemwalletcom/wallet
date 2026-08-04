// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import Primitives

public protocol FeeRateProviding: Sendable {
    func rates(for type: TransferDataType) async throws -> [FeeRate]
}

public struct FeeRateService: FeeRateProviding {
    private let service: any ChainFeeRateFetchable

    public init(
        service: any ChainFeeRateFetchable,
    ) {
        self.service = service
    }

    public func rates(for type: TransferDataType) async throws -> [FeeRate] {
        try await service.feeRates(type: type)
    }
}
