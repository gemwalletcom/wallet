// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import StakeService

public struct MockStakeService: StakeServiceable {
    public let stakeApr: Double?

    public init(stakeApr: Double? = .none) {
        self.stakeApr = stakeApr
    }

    public func stakeApr(assetId _: AssetId) throws -> Double? {
        stakeApr
    }

    public func update(walletId _: WalletId, chain _: Chain, address _: String) async throws {
        //
    }
}
