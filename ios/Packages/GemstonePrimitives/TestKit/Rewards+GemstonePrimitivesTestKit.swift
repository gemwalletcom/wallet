// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemDurationPart
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemListRow
import struct Gemstone.GemListSection
import enum Gemstone.GemRewardsAction
import struct Gemstone.GemRewardsRedemption
import struct Gemstone.GemRewardsState
import struct Gemstone.RedemptionResult
import enum Gemstone.RedemptionStatus
import struct Gemstone.ReferralAllowance
import struct Gemstone.ReferralQuota
import struct Gemstone.RewardRedemption
import struct Gemstone.RewardRedemptionOption
import struct Gemstone.Rewards
import enum Gemstone.RewardStatus
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

public extension GemRewardsState {
    static func mock(
        actions: [GemRewardsAction] = [.share, .useReferralCode],
        errorNotice: GemListRow? = nil,
        statusNotice: GemListRow? = nil,
        sections: [GemListSection] = [],
        inviteRewardPoints: GemFormattedNumber = .mock(value: 100, display: .number(precision: .fraction(min: 0, max: 0)), notation: .plain),
        referralCode: String? = "test123",
        referralLink: String? = "https://gemwallet.com/join?code=test123",
        usedReferralCode: String? = nil,
        redemptions: [GemRewardsRedemption] = [],
    ) -> GemRewardsState {
        GemRewardsState(
            actions: actions,
            errorNotice: errorNotice,
            statusNotice: statusNotice,
            sections: sections,
            inviteRewardPoints: inviteRewardPoints,
            referralCode: referralCode,
            referralLink: referralLink,
            usedReferralCode: usedReferralCode,
            redemptions: redemptions,
        )
    }
}

public extension GemRewardsRedemption {
    static func mock(
        option: RewardRedemptionOption = .mock(),
        canRedeem: Bool = true,
        points: GemFormattedNumber = .mock(value: 100, display: .number(precision: .fraction(min: 0, max: 0)), notation: .plain),
        value: GemFormattedNumber = .mock(),
    ) -> GemRewardsRedemption {
        GemRewardsRedemption(option: option, canRedeem: canRedeem, points: points, value: value)
    }
}
