package com.gemwallet.android.features.earn.delegation.models

import androidx.compose.runtime.Stable
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.CryptoFormattedUIModel
import com.gemwallet.android.ui.models.FiatFormattedUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemValidatorRow
import uniffi.gemstone.GemValueStyle

@Stable
class HeadDelegationInfo(private val delegation: Delegation, private val assetInfo: AssetInfo, override val currency: Currency, private val validator: GemValidatorRow) :
    CryptoFormattedUIModel,
    FiatFormattedUIModel {

    val iconUrl: String
        get() = validator.imageUrl

    val iconPlaceholder: String
        get() = validator.placeholder

    override val cryptoAmount: Double by lazy {
        Crypto(delegation.base.balance).value(asset.decimals).toDouble()
    }

    override val fiat: Double? by lazy { CryptoFiatConverter.fiatValue(Crypto(delegation.base.balance), asset.decimals, assetInfo.price?.price?.price) }

    override val asset: Asset
        get() = assetInfo.asset

    override val cryptoFormatted: String by lazy { ValueFormatter(style = GemValueStyle.AUTO).string(delegation.base.balance, asset) }

    override val fiatFormatted: String by lazy { super<FiatFormattedUIModel>.fiatFormatted }
}
