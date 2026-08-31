// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import protocol Gemstone.GemStakeServiceProtocol

public struct AmountService: Sendable {
    let stakeService: any GemStakeServiceProtocol
    public let amountService: GemAmountService

    public init(stakeService: any GemStakeServiceProtocol, amountService: GemAmountService) {
        self.stakeService = stakeService
        self.amountService = amountService
    }
}
