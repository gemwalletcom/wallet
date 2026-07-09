// Copyright (c). Gem Wallet. All rights reserved.

import GemAPI
import Primitives
import ScanService

public extension ScanService {
    static func mock(
        getScanTransaction: @escaping @Sendable (ScanTransactionPayload) async throws -> ScanTransaction = { _ in
            ScanTransaction(isMalicious: false, isMemoRequired: false)
        }
    ) -> ScanService {
        ScanService(apiService: ScanAPIServiceMock(getScanTransaction: getScanTransaction))
    }
}

private struct ScanAPIServiceMock: GemAPIScanService {
    let getScanTransaction: @Sendable (ScanTransactionPayload) async throws -> ScanTransaction

    func getScanTransaction(payload: ScanTransactionPayload) async throws -> ScanTransaction {
        try await getScanTransaction(payload)
    }
}
