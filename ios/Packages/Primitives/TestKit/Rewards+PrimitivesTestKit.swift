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
        disableReason: String? = nil,
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

public extension RedemptionResult {
    static func mock(
        option: RewardRedemptionOption = RewardRedemptionOption(id: "option", redemptionType: .asset, points: 100, asset: nil, value: "1", remaining: nil),
        status: RedemptionStatus = .completed,
    ) -> RedemptionResult {
        RedemptionResult(
            redemption: RewardRedemption(id: 1, option: option, status: status, transactionId: nil, createdAt: .now),
        )
    }
}
