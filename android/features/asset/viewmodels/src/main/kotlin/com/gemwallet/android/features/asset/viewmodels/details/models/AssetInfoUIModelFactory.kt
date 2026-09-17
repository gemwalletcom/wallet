package com.gemwallet.android.features.asset.viewmodels.details.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.banner.BannerRow
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.getTotalAmount
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.banner.uiModel
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.wallet.core.primitives.Currency
import dagger.hilt.android.qualifiers.ApplicationContext
import java.math.BigInteger
import javax.inject.Inject
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemValueStyle

class AssetInfoUIModelFactory @Inject constructor(@ApplicationContext private val context: Context) {

    fun create(chainAssetInfo: ChainAssetInfo, details: GemAssetDetails, banners: List<BannerRow>): AssetInfoUIModel {
        val assetInfo = chainAssetInfo.assetInfo
        val feeAssetInfo = chainAssetInfo.feeAssetInfo
        val asset = assetInfo.asset
        val balances = assetInfo.balance
        val price = assetInfo.price?.price?.price ?: 0.0
        val currency = assetInfo.price?.currency ?: Currency.USD
        val currencyFormatter = CurrencyFormatter(currency = currency)
        val valueFormatter = ValueFormatter(style = GemValueStyle.AUTO)
        val fiatTotal = if (balances.fiatTotalAmount == 0.0) "" else currencyFormatter.string(balances.fiatTotalAmount)
        return AssetInfoUIModel(
            assetInfo = assetInfo,
            name = details.title,
            iconUrl = asset.id.iconModel(),
            priceValue = if (price == 0.0) "" else currencyFormatter.string(price),
            priceDayChanges = assetInfo.price?.price?.priceChangePercentage24h.formatAsPercentage(),
            priceChangedType = assetInfo.price?.price?.priceChangePercentage24h.tone(),
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
            priceAlertMenu = details.state.priceAlert.menu(),
            emptyTransactions = details.state.emptyTransactionsAction.emptyTransactions(),
            banners = banners.map { it.uiModel(context) },
            pinListItem = ListItemModel(
                title = context.getString(if (assetInfo.metadata.isPinned) R.string.common_unpin else R.string.common_pin),
                image = ListItemImage.Symbol(ListItemSymbol.Pin),
            ),
            addListItem = ListItemModel(title = context.getString(R.string.asset_add_to_wallet), image = ListItemImage.Symbol(ListItemSymbol.AddCircle)),
            priceListItem = ListItemModel(title = context.getString(R.string.asset_price), subtitle = if (price == 0.0) "" else currencyFormatter.string(price)),
            priceAlertsListItem = ListItemModel(title = context.getString(R.string.settings_price_alerts_title), subtitle = details.state.priceAlertsCount.toString()),
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
                is GemBalanceRow.Available -> balance(row, row.viewType(), text(row.value))
                is GemBalanceRow.Staked -> balance(
                    row,
                    row.viewType(),
                    if (row.value == BigInteger.ZERO) {
                        "APR ${(assetInfo.metadata.stakingApr ?: 0.0).formatAsPercentage(style = GemPercentageStyle.UNSIGNED)}"
                    } else {
                        text(row.value)
                    },
                )
                is GemBalanceRow.PendingUnconfirmed -> balance(row, row.viewType(), text(row.value), info = InfoSheetEntity.PendingUnconfirmedBalanceInfo)
                is GemBalanceRow.Reserved -> balance(row, row.viewType(), text(row.value), url = row.url)
                is GemBalanceRow.Earn -> balance(
                    row,
                    row.viewType(),
                    if (row.value == BigInteger.ZERO) {
                        "APR ${(assetInfo.metadata.earnApr ?: 0.0).formatAsPercentage(style = GemPercentageStyle.UNSIGNED)}"
                    } else {
                        text(row.value)
                    },
                )
            }
        }
    }

    private fun balance(row: GemBalanceRow, type: AssetInfoUIModel.BalanceViewType, value: String, url: String? = null, info: InfoSheetEntity? = null) = AssetInfoUIModel.BalanceUIModel(
        type = type,
        url = url,
        model = ListItemModel(title = context.getString(row.title().titleRes()), subtitle = value, info = info),
    )
}
