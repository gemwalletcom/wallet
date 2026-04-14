// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Rewards {
    static func mock(
        code: String? = "test123",
        referralCount: Int32 = 5,
        points: Int32 = 0,
        usedReferralCode: String? = nil,
        status: RewardStatus = .verified,
        verifyAfter: Date? = .none,
        redemptionOptions: [RewardRedemptionOption] = [],
        disableReason: String? = nil
    ) -> Rewards {
        Rewards(
            code: code,
            referralCount: referralCount,
            points: points,
            usedReferralCode: usedReferralCode,
            status: status,
            verifyAfter: verifyAfter,
            redemptionOptions: redemptionOptions,
            disableReason: disableReason,
        )
    }
}
