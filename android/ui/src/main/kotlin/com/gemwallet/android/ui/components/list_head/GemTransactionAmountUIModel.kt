package com.gemwallet.android.ui.components.list_head

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemValueStyle

fun GemTransactionAmount.amountText(): String = sign.amount(value, asset.decimals.toUInt(), asset.symbol, GemValueStyle.AUTO).text()

fun GemTransactionAmount.valueText(): String = ValueFormatter(style = GemValueStyle.AUTO).string(value, asset.toPrimitives())

fun GemTransactionAmount.fiat(currency: Currency): String? = price?.let {
    CryptoFiatConverter.toFiatString(Crypto(value), asset.toPrimitives().decimals, it.price, currency)
}

fun GemTransactionAmount.priceValue(currency: Currency): AssetPriceValue = AssetPriceValue(
    asset = asset.toPrimitives(),
    price = price?.let { AssetPriceInfo(currency, it.toPrimitives()) },
)
