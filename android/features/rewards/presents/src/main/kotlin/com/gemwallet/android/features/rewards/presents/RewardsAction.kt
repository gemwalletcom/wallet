package com.gemwallet.android.features.rewards.presents

import com.gemwallet.android.features.rewards.viewmodels.models.RewardsSheetType
import uniffi.gemstone.GemRewardsRedemption

internal sealed interface RewardsAction {
    data object Refresh : RewardsAction
    data object Close : RewardsAction
    data object SelectWallet : RewardsAction
    data object InviteFriends : RewardsAction
    data object CodeHandled : RewardsAction
    data object DismissSheet : RewardsAction
    data class ShowSheet(val sheet: RewardsSheetType) : RewardsAction
    data class CreateReferral(val username: String) : RewardsAction
    data class UseCode(val code: String) : RewardsAction
    data class Redeem(val redemption: GemRewardsRedemption) : RewardsAction
}
