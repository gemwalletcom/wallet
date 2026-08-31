package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.cases.GetBuyQuoteUrl
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import uniffi.gemstone.GemFiatService
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.FiatQuoteUrl

class GetBuyQuoteUrlImpl(
    private val fiatService: GemFiatService,
) : GetBuyQuoteUrl {

    override suspend fun invoke(quoteId: String, walletId: WalletId): String? {
        return try {
            fiatService.getQuoteUrl(
                walletId = walletId.id,
                quoteId = quoteId,
            ).decodeJson<FiatQuoteUrl>().redirectUrl
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
            null
        }
    }
}
