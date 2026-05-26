package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.referral.coordinators.UseReferralCode
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.domains.referral.values.ReferralError
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.referralChain
import com.wallet.core.primitives.AuthenticatedRequest
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ReferralCode
import com.wallet.core.primitives.Wallet

class UseReferralCodeImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val getAuthPayload: GetAuthPayload,
) : UseReferralCode {


    override suspend fun useReferralCode(code: String, wallet: Wallet): Boolean {
        val account = wallet.getAccount(Chain.referralChain) ?: throw ReferralError.BadWallet
        val auth = getAuthPayload.getAuthPayload(wallet, account.chain)
        gemDeviceApiClient.useReferralCode(
            walletId = wallet.id,
            body = AuthenticatedRequest(
                auth = auth,
                data = ReferralCode(code)
            )
        )
        return true
    }
}
