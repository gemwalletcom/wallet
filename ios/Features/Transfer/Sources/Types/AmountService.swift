// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol

public struct AmountService: Sendable {
    let stakeService: any GemStakeServiceProtocol

    public init(stakeService: any GemStakeServiceProtocol) {
        self.stakeService = stakeService
    }
}
