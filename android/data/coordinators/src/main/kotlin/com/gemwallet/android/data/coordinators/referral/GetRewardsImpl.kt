package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.referral.coordinators.GetRewards
import com.gemwallet.android.domains.referral.values.ReferralError
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class GetRewardsImpl(
    private val rewardsService: GemRewardsService,
) : GetRewards {
    override suspend fun getRewards(walletId: WalletId): Rewards {
        val response = rewardsService.getRewards(walletId.id).decodeJson<Rewards>()
        if (response.code == null) {
            throw ReferralError.NotCreated
        }
        return response
    }
}
