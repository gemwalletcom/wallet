package com.gemwallet.android.testkit

import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRewardsResult
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.ReferralAllowance
import uniffi.gemstone.ReferralQuota
import uniffi.gemstone.RewardStatus
import uniffi.gemstone.Rewards

fun mockGemRewardsResult(
    walletId: String,
    rewards: Rewards? =
        mockRewards(inviteRewardPoints = 100, status = RewardStatus.VERIFIED, referralAllowance = mockReferralAllowance(daily = mockReferralQuota(limit = 5, available = 5), weekly = mockReferralQuota(limit = 20, available = 20))),
    error: GemServiceException? = null,
) = GemRewardsResult(
    walletId = walletId,
    state = error?.let { GemLoadState.Error(it) } ?: GemLoadState.Data,
    rewards = rewards.takeIf { error == null },
)
