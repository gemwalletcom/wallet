package com.gemwallet.android.data.coordinators.referral

import com.gemwallet.android.application.referral.coordinators.GetRewards
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.domains.referral.values.ReferralError
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.WalletId

class GetRewardsImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : GetRewards {
    override suspend fun getRewards(walletId: WalletId): Rewards {
        val response = gemDeviceApiClient.getRewards(walletId.id)
        if (response?.code == null) {
            throw ReferralError.NotCreated
        }
        return response
    }
}
