package com.gemwallet.android.testkit

import uniffi.gemstone.GemRewardsState
import uniffi.gemstone.ReferralAllowance
import uniffi.gemstone.ReferralQuota
import uniffi.gemstone.RewardStatus
import uniffi.gemstone.Rewards

fun mockRewards(code: String? = null, usedReferralCode: String? = null, points: Int = 0, status: RewardStatus = RewardStatus.VERIFIED) = Rewards(
    code = code,
    inviteRewardPoints = 100,
    referralCount = 0,
    points = points,
    usedReferralCode = usedReferralCode,
    status = status,
    createdAt = 0,
    verifyAfter = null,
    redemptionOptions = emptyList(),
    disableReason = null,
    referralAllowance = ReferralAllowance(daily = ReferralQuota(limit = 5, available = 5), weekly = ReferralQuota(limit = 20, available = 20)),
)

fun mockGemRewardsState(referralCode: String? = null, usedReferralCode: String? = null, canUseReferralCode: Boolean = false, showsPendingActivation: Boolean = false) = GemRewardsState(
    hasReferralCode = referralCode != null,
    canInvite = referralCode != null,
    canUseReferralCode = canUseReferralCode,
    showsInfo = referralCode != null || usedReferralCode != null,
    errorNotice = null,
    statusNotice = null,
    showsPendingActivation = showsPendingActivation,
    canActivatePendingReferral = false,
    inviteRewardPoints = mockFormattedNumber(100.0),
    referralCode = referralCode,
    referralLink = referralCode?.let { "https://gemwallet.com/join?code=$it" },
    usedReferralCode = usedReferralCode,
    infoRows = emptyList(),
    redemptions = emptyList(),
)
