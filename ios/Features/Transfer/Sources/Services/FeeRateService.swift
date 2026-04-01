// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import Primitives

protocol FeeRateProviding: Sendable {
    func rates(for type: TransferDataType) async throws -> [FeeRate]
}

struct FeeRateService: FeeRateProviding {
    private let service: any ChainFeeRateFetchable

    init(
        service: any ChainFeeRateFetchable,
    ) {
        self.service = service
    }

    func rates(for type: TransferDataType) async throws -> [FeeRate] {
        try await service.feeRates(type: type)
    }
}
