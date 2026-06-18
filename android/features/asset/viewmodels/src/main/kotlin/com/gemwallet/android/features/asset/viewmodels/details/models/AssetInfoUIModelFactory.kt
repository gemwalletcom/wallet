package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.isStaked
import com.gemwallet.android.ext.type
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.getStackedAmount
import com.gemwallet.android.model.getTotalAmount
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.StakeChain
import uniffi.gemstone.Explorer

object AssetInfoUIModelFactory {

    fun create(chainAssetInfo: ChainAssetInfo, explorerName: String): AssetInfoUIModel {
        val assetInfo = chainAssetInfo.assetInfo
        val feeAssetInfo = chainAssetInfo.feeAssetInfo
        val asset = assetInfo.asset
        val balances = assetInfo.balance
        val price = assetInfo.price?.price?.price ?: 0.0
        val currency = assetInfo.price?.currency ?: Currency.USD
        val currencyFormatter = CurrencyFormatter(currency = currency)
        val valueFormatter = ValueFormatter(style = ValueFormatter.Style.Auto)
        val fiatTotal = if (balances.fiatTotalAmount == 0.0) "" else currencyFormatter.string(balances.fiatTotalAmount)

        return AssetInfoUIModel(
            assetInfo = assetInfo,
            name = assetName(asset),
            iconUrl = asset.id.getIconUrl(),
            priceValue = if (price == 0.0) "" else currencyFormatter.string(price),
            priceDayChanges = assetInfo.price?.price?.priceChangePercentage24h.formatAsPercentage(),
            priceChangedType = assetInfo.price?.price?.priceChangePercentage24h.toValueDirection(),
            tokenType = asset.type,
            isBuyEnabled = assetInfo.metadata?.isBuyEnabled == true,
            isSwapEnabled = assetInfo.metadata?.isSwapEnabled == true,
            explorerName = explorerName,
            explorerAddressUrl = assetInfo.owner?.address?.let {
                Explorer(asset.chain.string).getAddressUrl(explorerName, it)
            },
            explorerTokenUrl = asset.id.tokenId?.let {
                Explorer(asset.chain.string).getTokenUrl(explorerName, it)
            },
            accountInfoUIModel = AssetInfoUIModel.AccountInfoUIModel(
                walletType = assetInfo.walletType,
                totalBalance = valueFormatter.string(balances.balance.getTotalAmount(), balances.asset),
                totalFiat = fiatTotal,
                owner = assetInfo.owner?.address ?: "",
                balanceMetadata = feeAssetInfo.balance.metadata,
                hasBalanceDetails = StakeChain.isStaked(asset.id.chain) || balances.balanceAmount.reserved != 0.0,
                available = formatAvailable(assetInfo, valueFormatter),
                stake = formatStake(assetInfo, valueFormatter),
                reserved = formatReserved(assetInfo, valueFormatter),
            ),
        )
    }

    private fun assetName(asset: Asset): String =
        if (asset.type == AssetType.NATIVE) asset.id.chain.asset().name else asset.name

    private fun formatAvailable(assetInfo: AssetInfo, formatter: ValueFormatter): String {
        val balances = assetInfo.balance
        return if (balances.balanceAmount.available != balances.totalAmount) {
            formatter.string(balances.balance.available.toBigInteger(), balances.asset)
        } else {
            ""
        }
    }

    private fun formatStake(assetInfo: AssetInfo, formatter: ValueFormatter): String {
        val asset = assetInfo.asset
        if (asset.id.type() != AssetSubtype.NATIVE || !StakeChain.isStaked(asset.id.chain)) {
            return ""
        }
        val balances = assetInfo.balance
        return if (balances.balanceAmount.getStackedAmount() == 0.0) {
            "APR ${(assetInfo.stakeApr ?: 0.0).formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)}"
        } else {
            formatter.string(balances.balance.getStackedAmount(), balances.asset)
        }
    }

    private fun formatReserved(assetInfo: AssetInfo, formatter: ValueFormatter): String {
        val balances = assetInfo.balance
        return if (balances.balanceAmount.reserved != 0.0) {
            formatter.string(balances.balance.reserved.toBigInteger(), balances.asset)
        } else {
            ""
        }
    }
}
