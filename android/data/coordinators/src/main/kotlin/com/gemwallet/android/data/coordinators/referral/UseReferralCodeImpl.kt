package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.referral.coordinators.UseReferralCode
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.toJson

class UseReferralCodeImpl(
    private val rewardsService: GemRewardsService,
) : UseReferralCode {


    override suspend fun useReferralCode(code: String, wallet: Wallet): Boolean {
        rewardsService.useReferralCode(
            wallet = wallet.toJson(),
            code = code,
        )
        return true
    }
}
