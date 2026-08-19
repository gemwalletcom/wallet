package com.gemwallet.android.features.referral.viewmodels

import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.RewardStatus
import com.wallet.core.primitives.Rewards

data class RewardsUIState(
    val canInvite: Boolean,
    val isUnverified: Boolean,
    val hasPendingReferral: Boolean,
    val canActivatePendingReferral: Boolean,
) {
    val buttonState: ButtonState
        get() = buttonState(enabled = canActivatePendingReferral)

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
                canInvite = rewards.status == RewardStatus.Verified || rewards.status == RewardStatus.Trusted || rewards.status == RewardStatus.Attribution,
                isUnverified = rewards.code != null && rewards.status == RewardStatus.Unverified && verifyAfter == null,
                hasPendingReferral = verifyAfter != null,
                canActivatePendingReferral = verifyAfter != null && System.currentTimeMillis() > verifyAfter,
            )
        }
    }
}
