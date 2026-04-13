package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive

class GetBuyQuoteUrlImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : GetBuyQuoteUrl {

    override suspend fun invoke(quoteId: String, walletId: WalletId): String? {
        return try {
            gemDeviceApiClient.getFiatQuoteUrl(
                walletId = walletId.id,
                quoteId = quoteId,
            )?.redirectUrl
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
            null
        }
    }
}
