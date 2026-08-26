// Copyright (c). Gem Wallet. All rights reserved.

import GemAPI
import Primitives

public struct ScanService: Sendable {
    private let apiService: any GemAPIScanService

    public init(apiService: any GemAPIScanService) {
        self.apiService = apiService
    }

    public func getScanTransaction(payload: ScanTransactionPayload) async -> ScanTransaction? {
        try? await apiService.getScanTransaction(payload: payload)
    }
}
