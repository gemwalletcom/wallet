package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.referral.coordinators.CreateReferral
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class CreateReferralImpl(
    private val rewardsService: GemRewardsService,
    private val getAuthPayload: GetAuthPayload
) : CreateReferral {


    override suspend fun createReferral(code: String, wallet: Wallet): Rewards {
        val authPayload = getAuthPayload.getAuthPayload(wallet)
        return rewardsService.createReferral(
            walletId = wallet.id.id,
            auth = authPayload.toJson(),
            code = code,
        ).decodeJson<Rewards>()
    }
}
