package com.gemwallet.android.features.referral.viewmodels

import com.wallet.core.primitives.RewardStatus
import com.wallet.core.primitives.Rewards

data class RewardsUIState(
    val canInvite: Boolean,
    val isUnverified: Boolean,
    val hasPendingReferral: Boolean,
    val canActivatePendingReferral: Boolean,
) {
    companion object {
        fun from(rewards: Rewards?): RewardsUIState {
            if (rewards == null) {
                return RewardsUIState(
                    canInvite = false,
                    isUnverified = false,
                    hasPendingReferral = false,
                    canActivatePendingReferral = false,
                )
            }
            val verifyAfter = rewards.verifyAfter
            return RewardsUIState(
                canInvite = rewards.status == RewardStatus.Verified || rewards.status == RewardStatus.Trusted,
                isUnverified = rewards.code != null && rewards.status == RewardStatus.Unverified && verifyAfter == null,
                hasPendingReferral = verifyAfter != null,
                canActivatePendingReferral = verifyAfter != null && System.currentTimeMillis() > verifyAfter,
            )
        }
    }
}
