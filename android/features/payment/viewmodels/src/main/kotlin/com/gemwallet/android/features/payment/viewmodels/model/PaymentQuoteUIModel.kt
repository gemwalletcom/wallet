package com.gemwallet.android.features.payment.viewmodels.model

import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.getSupportIconUrl
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.GemPaymentPrice
import uniffi.gemstone.GemPaymentQuote

private val amountFormatter = ValueFormatter(ValueFormatter.Style.Short)
private val priceFormatter = ValueFormatter(ValueFormatter.Style.Full)

data class PaymentQuoteUIModel(
    val id: String,
    val symbol: String,
    val amount: String,
    val iconUrl: String?,
    val supportIconUrl: String?,
) {
    val amountText: String get() = "$amount $symbol"
}

fun GemPaymentQuote.toUIModel(): PaymentQuoteUIModel {
    val assetId = amount.assetId.toAssetId()
    return PaymentQuoteUIModel(
        id = id,
        symbol = amount.symbol,
        amount = amountFormatter.string(Crypto(amount.value).value(amount.decimals)),
        iconUrl = assetId?.getIconUrl(),
        supportIconUrl = assetId?.getSupportIconUrl(),
    )
}

fun GemPaymentPrice.toPriceText(): String = priceFormatter.string(Crypto(value).value(decimals), currency = symbol)
