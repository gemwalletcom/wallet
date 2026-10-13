package com.gemwallet.android.features.rewards.presents

import uniffi.gemstone.GemRewardsRedemption

internal sealed interface RewardsAction {
    data object Refresh : RewardsAction
    data object Close : RewardsAction
    data object SelectWallet : RewardsAction
    data object InviteFriends : RewardsAction
    data object CodeHandled : RewardsAction
    data class ActivateCode(val code: String) : RewardsAction
    data class Redeem(val redemption: GemRewardsRedemption) : RewardsAction
}
