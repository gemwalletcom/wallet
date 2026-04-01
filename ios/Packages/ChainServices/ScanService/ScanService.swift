// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import Primitives

public struct ScanService: Sendable {
    private let gatewayService: GatewayService

    public init(gatewayService: GatewayService) {
        self.gatewayService = gatewayService
    }

    public func getScanTransaction(chain: Primitives.Chain, input: TransactionPreloadInput) async -> ScanTransaction? {
        try? await gatewayService.transactionScan(chain: chain, input: input)
    }
}
