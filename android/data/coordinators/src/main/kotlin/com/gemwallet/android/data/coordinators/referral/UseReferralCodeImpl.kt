package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.referral.cases.UseReferralCode
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class UseReferralCodeImpl(
    private val rewardsService: GemRewardsService,
) : UseReferralCode {


    override suspend fun useReferralCode(code: String, wallet: Wallet): Boolean = withContext(Dispatchers.IO) {
        rewardsService.useReferralCode(
            wallet = wallet.toJson(),
            code = code,
        )
        true
    }
}
