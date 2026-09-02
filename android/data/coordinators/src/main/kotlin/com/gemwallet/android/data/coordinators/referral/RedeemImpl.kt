package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.referral.cases.Redeem
import com.gemwallet.android.domains.referral.values.ReferralError
import com.wallet.core.primitives.RedemptionResult
import com.wallet.core.primitives.RewardRedemptionOption
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class RedeemImpl(
    private val rewardsService: GemRewardsService,
) : Redeem {

    override suspend fun redeem(wallet: Wallet, rewards: Rewards, option: RewardRedemptionOption): RedemptionResult {
        if (rewards.points < option.points) {
            throw ReferralError.InsufficientPoints
        }
        return withContext(Dispatchers.IO) {
            rewardsService.redeem(wallet = wallet.toJson(), redemptionId = option.id).decodeJson()
        }
    }
}
