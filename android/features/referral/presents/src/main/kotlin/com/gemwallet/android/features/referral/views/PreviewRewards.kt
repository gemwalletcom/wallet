package com.gemwallet.android.features.referral.views

import uniffi.gemstone.ReferralAllowance
import uniffi.gemstone.ReferralQuota
import uniffi.gemstone.RewardStatus
import uniffi.gemstone.Rewards

internal fun previewRewards(
    code: String? = null,
    referralCount: Int = 0,
    points: Int = 0,
    usedReferralCode: String? = null,
    status: RewardStatus = RewardStatus.PENDING,
    verifyAfter: Long? = null,
    disableReason: String? = null,
) = Rewards(
    code = code,
    inviteRewardPoints = 100,
    referralCount = referralCount,
    points = points,
    usedReferralCode = usedReferralCode,
    status = status,
    createdAt = 0L,
    verifyAfter = verifyAfter,
    redemptionOptions = emptyList(),
    disableReason = disableReason,
    referralAllowance = ReferralAllowance(
        daily = ReferralQuota(limit = 5, available = 5),
        weekly = ReferralQuota(limit = 20, available = 20),
    ),
)
