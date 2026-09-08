package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toAssetPriceInfo
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.AssetPrice
import uniffi.gemstone.GemFeeAsset
import java.math.BigDecimal
import java.math.BigInteger

data class FeeAssetUIModel(
    val asset: Asset,
    val price: AssetPriceInfo?,
    val available: BigInteger,
) {
    val priceValue: AssetPriceValue by lazy { AssetPriceValue(asset, price) }
    val isZeroBalance: Boolean get() = available.signum() == 0
    val balance: String by lazy { ValueFormatter(style = ValueFormatter.Style.Short).string(amount, asset.symbol) }
    val equivalent: String by lazy { priceValue.formatFiat(priceValue.calculateFiat(amount)) }

    private val amount: BigDecimal by lazy { Crypto(available).value(asset.decimals) }

    companion object {
        fun from(asset: Asset, balance: GemAssetBalance, price: AssetPrice?, currency: Currency) = FeeAssetUIModel(
            asset = asset,
            price = price?.toAssetPriceInfo(currency),
            available = balance.available,
        )
    }
}

fun GemFeeAsset.toFeeAssetUIModel(currency: Currency): FeeAssetUIModel = FeeAssetUIModel.from(asset.toPrimitives(), balance, price, currency)
