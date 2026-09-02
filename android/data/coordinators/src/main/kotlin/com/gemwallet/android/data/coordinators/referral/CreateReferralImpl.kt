package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.referral.cases.CreateReferral
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class CreateReferralImpl(
    private val rewardsService: GemRewardsService,
) : CreateReferral {


    override suspend fun createReferral(code: String, wallet: Wallet): Rewards = withContext(Dispatchers.IO) {
        rewardsService.createReferral(
            wallet = wallet.toJson(),
            code = code,
        ).decodeJson<Rewards>()
    }
}
