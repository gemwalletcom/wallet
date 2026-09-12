package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.getTotalAmount
import com.gemwallet.android.model.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Currency
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemBalanceRow
import javax.inject.Inject
import java.math.BigInteger

class AssetInfoUIModelFactory @Inject constructor(
    @param:EarnAvailable private val earnAvailable: Boolean,
) {

    fun create(chainAssetInfo: ChainAssetInfo, details: GemAssetDetails): AssetInfoUIModel {
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
            name = details.title,
            iconUrl = asset.id.iconModel(),
            priceValue = if (price == 0.0) "" else currencyFormatter.string(price),
            priceDayChanges = assetInfo.price?.price?.priceChangePercentage24h.formatAsPercentage(),
            priceChangedType = assetInfo.price?.price?.priceChangePercentage24h.toValueDirection(),
            tokenType = asset.type,
            isBuyEnabled = assetInfo.metadata.isBuyEnabled,
            isSwapEnabled = assetInfo.metadata.isSwapEnabled,
            swapPayAssetId = details.swapPair.payAssetId.toAssetId(),
            swapReceiveAssetId = details.swapPair.receiveAssetId?.toAssetId(),
            explorerName = details.explorerName,
            explorerAddressUrl = details.addressLink?.link,
            explorerTokenUrl = details.tokenLink?.link,
            verificationStatus = details.verificationStatus?.toPrimitives(),
            networkDestination = details.networkDestination,
            shareUrl = details.shareUrl,
            detailsState = details.state,
            accountInfoUIModel = AssetInfoUIModel.AccountInfoUIModel(
                totalBalance = valueFormatter.string(balances.balance.getTotalAmount(), balances.asset),
                totalFiat = fiatTotal,
                owner = assetInfo.owner?.address ?: "",
                balances = balanceRows(assetInfo, valueFormatter),
                balanceMetadata = feeAssetInfo.balance.metadata,
            ),
        )
    }

    private fun balanceRows(assetInfo: AssetInfo, formatter: ValueFormatter): List<AssetInfoUIModel.BalanceUIModel> {
        val asset = assetInfo.asset
        val text = { value: BigInteger -> formatter.string(value, asset) }
        return assetInfo.balance.toGem().detailRows(asset.chain.string, assetInfo.metadata.isStakeEnabled).mapNotNull { row ->
            when (row) {
                is GemBalanceRow.Available -> AssetInfoUIModel.BalanceUIModel(AssetInfoUIModel.BalanceViewType.Available, text(row.value))
                is GemBalanceRow.Staked -> AssetInfoUIModel.BalanceUIModel(
                    AssetInfoUIModel.BalanceViewType.Stake,
                    if (row.value == BigInteger.ZERO) {
                        "APR ${(assetInfo.metadata.stakingApr ?: 0.0).formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)}"
                    } else {
                        text(row.value)
                    },
                )
                is GemBalanceRow.PendingUnconfirmed -> AssetInfoUIModel.BalanceUIModel(AssetInfoUIModel.BalanceViewType.PendingUnconfirmed, text(row.value))
                is GemBalanceRow.Reserved -> AssetInfoUIModel.BalanceUIModel(AssetInfoUIModel.BalanceViewType.Reserved, text(row.value), row.url)
                is GemBalanceRow.Earn -> if (!earnAvailable) null else AssetInfoUIModel.BalanceUIModel(
                    AssetInfoUIModel.BalanceViewType.Earn,
                    if (row.value == BigInteger.ZERO) {
                        "APR ${(assetInfo.metadata.earnApr ?: 0.0).formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)}"
                    } else {
                        text(row.value)
                    },
                )
            }
        }
    }
}
