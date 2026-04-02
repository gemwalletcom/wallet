package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.feeUnitType
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignMode
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.FeeUnitType

data class FeeRateUIModel(
    val fee: Fee,
    val feeAssetInfo: AssetInfo? = null,
) {

    val price: String get() {
        val asset = fee.feeAssetId.chain.asset()
        val feeUnitType = asset.chain.feeUnitType()
        val decimals = when (feeUnitType) {
            FeeUnitType.SatVb -> 0
            FeeUnitType.Gwei -> 9
            FeeUnitType.Native -> asset.decimals
            null -> 0
        }
        val symbol = when (feeUnitType) {
            FeeUnitType.SatVb, FeeUnitType.Gwei -> feeUnitType.string
            FeeUnitType.Native -> asset.symbol
            null -> ""
        }
        val value = when (asset.chain) {
            Chain.Solana -> fee.amount
            else -> when (fee) {
                is Fee.Eip1559 -> fee.maxGasPrice
                is Fee.Plain -> fee.amount
                is Fee.Regular -> fee.maxGasPrice
                is Fee.Solana -> fee.maxGasPrice
            }
        }
        val formattedValue = Crypto(value).format(decimals, symbol, 8, -1, SignMode.NoSign, true)
        return formattedValue
    }

    val fiatValue: String get() {
        val assetInfo = feeAssetInfo ?: return ""
        val priceInfo = assetInfo.price ?: return ""
        val currency = priceInfo.currency
        val feeAmount = Crypto(fee.amount)
        val fiat = feeAmount.convert(assetInfo.asset.decimals, priceInfo.price.price)
        return currency.format(fiat.atomicValue, dynamicPlace = true)
    }

    val priority = fee.priority

    val emoji: String get() = when (priority) {
        FeePriority.Slow -> "\uD83D\uDC22"
        FeePriority.Normal -> "\uD83D\uDC8E"
        FeePriority.Fast -> "\uD83D\uDE80"
    }
}
