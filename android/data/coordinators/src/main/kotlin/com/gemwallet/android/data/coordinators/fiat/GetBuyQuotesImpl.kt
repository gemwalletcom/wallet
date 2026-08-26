package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import okio.IOException
import uniffi.gemstone.GemFiatService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class GetBuyQuotesImpl(
    private val fiatService: GemFiatService,
) : GetBuyQuotes {

    override suspend fun invoke(
        walletId: WalletId,
        asset: Asset,
        type: FiatQuoteType,
        fiatCurrency: String,
        amount: Double,
    ): List<FiatQuote> {
        return try {
            fiatService.getQuotes(
                walletId = walletId.id,
                quoteType = type.toJson(),
                assetId = asset.id.toIdentifier(),
                amount = amount,
                currency = fiatCurrency,
            ).map { it.decodeJson<FiatQuote>() }
        } catch (err: Exception) {
            currentCoroutineContext().ensureActive()
            throw Exception("Quotes not found", err)
        }
    }
}
