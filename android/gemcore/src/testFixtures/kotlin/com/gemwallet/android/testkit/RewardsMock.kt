package com.gemwallet.android.testkit

import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRewardsAction
import uniffi.gemstone.GemRewardsResult
import uniffi.gemstone.GemRewardsState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.ReferralAllowance
import uniffi.gemstone.ReferralQuota
import uniffi.gemstone.RewardStatus
import uniffi.gemstone.Rewards

fun mockRewards(code: String? = null, usedReferralCode: String? = null, points: Int = 0, status: RewardStatus = RewardStatus.VERIFIED, verifyAfter: Long? = null) = Rewards(
    code = code,
    inviteRewardPoints = 100,
    referralCount = 0,
    points = points,
    usedReferralCode = usedReferralCode,
    status = status,
    createdAt = 0,
    verifyAfter = verifyAfter,
    redemptionOptions = emptyList(),
    disableReason = null,
    referralAllowance = ReferralAllowance(daily = ReferralQuota(limit = 5, available = 5), weekly = ReferralQuota(limit = 20, available = 20)),
    useReferralCodeUntil = null,
)

fun mockGemRewardsState(referralCode: String? = null, usedReferralCode: String? = null, canUseReferralCode: Boolean = false, showsPendingActivation: Boolean = false) = GemRewardsState(
    actions = listOfNotNull(
        if (referralCode == null) GemRewardsAction.CreateCode else GemRewardsAction.Share,
        GemRewardsAction.UseReferralCode.takeIf { canUseReferralCode },
        usedReferralCode?.takeIf { showsPendingActivation }?.let { GemRewardsAction.ActivatePendingReferral(it, false) },
    ),
    errorNotice = null,
    statusNotice = null,
    sections = emptyList(),
    inviteRewardPoints = mockFormattedNumber(100.0),
    referralCode = referralCode,
    referralLink = referralCode?.let { "https://gemwallet.com/join?code=$it" },
    usedReferralCode = usedReferralCode,
    redemptions = emptyList(),
)

fun mockGemRewardsResult(walletId: String, rewards: Rewards? = mockRewards(), error: GemServiceException? = null) = GemRewardsResult(
    walletId = walletId,
    state = error?.let { GemLoadState.Error(it) } ?: GemLoadState.Data,
    rewards = rewards.takeIf { error == null },
)
