package com.gemwallet.android.ui.models

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.DelegationBase
import java.math.BigInteger
import uniffi.gemstone.GemValueStyle

open class BalanceInfoUIModel(
    override val asset: Asset,
    private val balance: BigInteger,
    val price: Double?,
    override val currency: Currency
) : CryptoFormattedUIModel, FiatFormattedUIModel {

    override val cryptoAmount: Double by lazy { Crypto(balance).value(asset.decimals).toDouble() }

    override val fiat: Double? by lazy {
        val price = price ?: return@lazy null
        if (price == 0.0) null else CryptoFiatConverter.toFiat(Crypto(balance), asset.decimals, price).atomicValue.toDouble()
    }
}

class RewardsInfoUIModel(
    assetInfo: AssetInfo,
    balance: BigInteger,
) : BalanceInfoUIModel(
    asset = assetInfo.asset,
    balance = balance,
    price = assetInfo.price?.price?.price,
    currency = assetInfo.price?.currency ?: Currency.USD,
) {
    override val cryptoFormatted: String by lazy { ValueFormatter(style = GemValueStyle.AUTO).string(balance, asset) }
}

class DelegationBalanceInfoUIModel(
    assetInfo: AssetInfo,
    delegation: DelegationBase,
) : BalanceInfoUIModel(
    asset = assetInfo.asset,
    balance = delegation.balance,
    price = assetInfo.price?.price?.price,
    currency = assetInfo.price?.currency ?: Currency.USD,
)