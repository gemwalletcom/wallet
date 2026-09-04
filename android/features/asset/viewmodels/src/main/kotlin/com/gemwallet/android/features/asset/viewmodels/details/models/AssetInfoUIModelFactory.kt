package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.getIconUrl
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
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.WalletType
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemSwapPairSuggestion
import javax.inject.Inject
import java.math.BigInteger

class AssetInfoUIModelFactory @Inject constructor() {

    fun create(
        chainAssetInfo: ChainAssetInfo,
        swapPair: GemSwapPairSuggestion,
        explorerName: String,
        walletType: WalletType,
        explorerAddressUrl: String?,
        explorerTokenUrl: String?,
        verificationStatus: VerificationStatus?,
        networkDestination: GemAssetNetworkDestination?,
        shareUrl: String,
    ): AssetInfoUIModel {
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
            isBuyEnabled = assetInfo.metadata.isBuyEnabled,
            isSwapEnabled = assetInfo.metadata.isSwapEnabled,
            swapPayAssetId = swapPair.payAssetId.toAssetId(),
            swapReceiveAssetId = swapPair.receiveAssetId?.toAssetId(),
            explorerName = explorerName,
            explorerAddressUrl = explorerAddressUrl,
            explorerTokenUrl = explorerTokenUrl,
            verificationStatus = verificationStatus,
            networkDestination = networkDestination?.let(::networkDestination),
            shareUrl = shareUrl,
            accountInfoUIModel = AssetInfoUIModel.AccountInfoUIModel(
                walletType = walletType,
                totalBalance = valueFormatter.string(balances.balance.getTotalAmount(), balances.asset),
                totalFiat = fiatTotal,
                owner = assetInfo.owner?.address ?: "",
                balances = balanceRows(assetInfo, valueFormatter),
                balanceMetadata = feeAssetInfo.balance.metadata,
            ),
        )
    }

    private fun networkDestination(destination: GemAssetNetworkDestination): AssetInfoUIModel.NetworkDestination? = when (destination) {
        is GemAssetNetworkDestination.Asset -> AssetInfoUIModel.NetworkDestination.Asset(destination.asset.toPrimitives().id)
        is GemAssetNetworkDestination.Assets -> AssetInfoUIModel.NetworkDestination.Assets(destination.chain.requireChain())
    }

    private fun assetName(asset: Asset): String =
        if (asset.type == AssetType.NATIVE) asset.id.chain.asset().name else asset.name

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
                is GemBalanceRow.Earn -> null
            }
        }
    }
}
