// Copyright (c). Gem Wallet. All rights reserved.

import GemAPI
import Primitives
import PrimitivesTestKit

public actor GemAPIWalletConfigurationServiceMock: GemAPIWalletConfigurationService {
    private let result: WalletConfigurationResult
    public private(set) var walletIds: [WalletId] = []

    public init(
        result: WalletConfigurationResult = WalletConfigurationResult(
            walletId: .mock(),
            configuration: WalletConfiguration(multiSignatureAccounts: []),
        ),
    ) {
        self.result = result
    }

    public func getWalletConfiguration(walletId: WalletId) async throws -> WalletConfigurationResult {
        walletIds.append(walletId)
        return result
    }
}
