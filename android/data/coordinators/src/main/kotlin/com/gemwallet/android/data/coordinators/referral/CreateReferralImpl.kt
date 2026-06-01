package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.referral.coordinators.CreateReferral
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.AuthenticatedRequest
import com.wallet.core.primitives.ReferralCode
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import java.io.IOException

class CreateReferralImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val getAuthPayload: GetAuthPayload
) : CreateReferral {


    override suspend fun createReferral(code: String, wallet: Wallet): Rewards {
        val authPayload = getAuthPayload.getAuthPayload(wallet)
        return gemDeviceApiClient.createReferral(
            walletId = wallet.id,
            body = AuthenticatedRequest(
                auth = authPayload,
                data = ReferralCode(
                    code = code
                )
            )
        ) ?: throw IOException("Request failed")
    }
}
