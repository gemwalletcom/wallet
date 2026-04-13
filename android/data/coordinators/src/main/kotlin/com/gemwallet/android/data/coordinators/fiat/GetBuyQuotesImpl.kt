package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import okio.IOException

class GetBuyQuotesImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : GetBuyQuotes {

    override suspend fun invoke(
        walletId: WalletId,
        asset: Asset,
        type: FiatQuoteType,
        fiatCurrency: String,
        amount: Double,
    ): List<FiatQuote> {
        return try {
            gemDeviceApiClient.getFiatQuotes(
                assetId = asset.id.toIdentifier(),
                amount = amount,
                currency = fiatCurrency,
                walletId = walletId.id,
                type = type.string,
            )?.quotes ?: throw IOException()
        } catch (err: Exception) {
            currentCoroutineContext().ensureActive()
            throw Exception("Quotes not found", err)
        }
    }
}
