// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.ReferralAllowance
import struct Gemstone.ReferralQuota
import enum Gemstone.RedemptionStatus
import struct Gemstone.RedemptionResult
import struct Gemstone.RewardRedemption
import struct Gemstone.RewardRedemptionOption
import enum Gemstone.RewardStatus
import struct Gemstone.Rewards
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
            inviteRewardPoints: 100,
            referralCount: referralCount,
            points: points,
            usedReferralCode: usedReferralCode,
            status: status,
            createdAt: 0,
            verifyAfter: verifyAfter,
            redemptionOptions: redemptionOptions,
            disableReason: disableReason,
            referralAllowance: .mock(),
        )
    }
}

public extension ReferralAllowance {
    static func mock(
        daily: ReferralQuota = ReferralQuota(limit: 5, available: 5),
        weekly: ReferralQuota = ReferralQuota(limit: 20, available: 20),
    ) -> ReferralAllowance {
        ReferralAllowance(daily: daily, weekly: weekly)
    }
}

public extension RewardRedemptionOption {
    static func mock(
        id: String = "option",
        points: Int32 = 100,
        value: BigUInt = 1,
    ) -> RewardRedemptionOption {
        RewardRedemptionOption(id: id, redemptionType: .asset, points: points, asset: nil, value: value, remaining: nil)
    }
}

public extension RedemptionResult {
    static func mock(
        option: RewardRedemptionOption = .mock(),
        status: RedemptionStatus = .completed,
    ) -> RedemptionResult {
        RedemptionResult(
            redemption: RewardRedemption(id: 1, option: option, status: status, transactionId: nil, createdAt: Date(timeIntervalSince1970: 0)),
        )
    }
}
