package com.gemwallet.android.features.payment.viewmodels.model

import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.getSupportIconUrl
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.PaymentPrice
import com.wallet.core.primitives.PaymentQuote

private val amountFormatter = ValueFormatter(ValueFormatter.Style.Short)
private val priceFormatter = ValueFormatter(ValueFormatter.Style.Full)

data class PaymentQuoteUIModel(
    val id: String,
    val name: String,
    val networkName: String,
    val symbol: String,
    val amount: String,
    val balance: String,
    val iconUrl: String?,
    val supportIconUrl: String?,
    val requiresVerification: Boolean,
) {
    val amountText: String get() = "$amount $symbol"
}

fun PaymentQuote.toUIModel(assetInfo: AssetInfo? = null): PaymentQuoteUIModel {
    val asset = assetInfo?.asset ?: assetId.chain.asset()
    return PaymentQuoteUIModel(
        id = id,
        name = asset.name,
        networkName = assetId.chain.networkName(),
        symbol = asset.symbol,
        amount = amountFormatter.string(Crypto(value).value(asset.decimals)),
        balance = assetInfo?.balanceText().orEmpty(),
        iconUrl = assetId.getIconUrl(),
        supportIconUrl = assetId.getSupportIconUrl(),
        requiresVerification = collectDataUrl != null,
    )
}

fun PaymentPrice.toPriceText(): String = priceFormatter.string(Crypto(value).value(decimals), currency = symbol)

private fun AssetInfo.balanceText(): String = amountFormatter.string(
    Crypto(balance.balance.available).value(asset.decimals),
    currency = asset.symbol,
)
