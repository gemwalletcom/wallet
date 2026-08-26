package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.referral.coordinators.UseReferralCode
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsService
import com.gemwallet.android.serializer.toJson

class UseReferralCodeImpl(
    private val rewardsService: GemRewardsService,
    private val getAuthPayload: GetAuthPayload,
) : UseReferralCode {


    override suspend fun useReferralCode(code: String, wallet: Wallet): Boolean {
        val auth = getAuthPayload.getAuthPayload(wallet)
        rewardsService.useReferralCode(
            walletId = wallet.id.id,
            auth = auth.toJson(),
            code = code,
        )
        return true
    }
}
