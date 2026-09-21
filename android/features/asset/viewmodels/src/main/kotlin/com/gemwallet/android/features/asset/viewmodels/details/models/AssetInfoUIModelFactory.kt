package com.gemwallet.android.features.asset.viewmodels.details.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.getTotalAmount
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.banner.uiModel
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.Currency
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemAssetBalanceRow
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemValueStyle
import javax.inject.Inject

class AssetInfoUIModelFactory @Inject constructor(@ApplicationContext private val context: Context) {

    fun create(chainAssetInfo: ChainAssetInfo, details: GemAssetDetails, banners: List<GemBannerRow>): AssetInfoUIModel {
        val assetInfo = chainAssetInfo.assetInfo
        val asset = assetInfo.asset
        val balances = assetInfo.balance
        val price = assetInfo.price?.price?.price ?: 0.0
        val currency = assetInfo.price?.currency ?: Currency.USD
        val currencyFormatter = CurrencyFormatter(currency = currency)
        val valueFormatter = ValueFormatter(style = GemValueStyle.AUTO)
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
            priceListItem = ListItemModel(title = context.getString(R.string.asset_price), subtitle = if (price == 0.0) "" else currencyFormatter.string(price)),
            sections = details.sections.map { section -> AssetInfoUIModel.SectionUIModel(section.title.titleRes(), section.rows.map { row(it) }) },
            accountInfoUIModel = AssetInfoUIModel.AccountInfoUIModel(
                totalBalance = valueFormatter.string(balances.balance.getTotalAmount(), balances.asset),
                totalFiat = details.fiatValue?.text().orEmpty(),
                owner = assetInfo.owner?.address ?: "",
            ),
        )
    }

    private fun row(row: GemAssetDetailRow): AssetInfoUIModel.RowUIModel = when (row) {
        GemAssetDetailRow.Price -> AssetInfoUIModel.RowUIModel.Price

        is GemAssetDetailRow.Network -> AssetInfoUIModel.RowUIModel.Network(row.name)

        is GemAssetDetailRow.Balance -> balance(row.row)

        is GemAssetDetailRow.Earn -> AssetInfoUIModel.RowUIModel.Earn(
            ListItemModel(title = context.getString(R.string.common_earn), subtitle = context.getString(R.string.stake_apr, row.apr?.text().orEmpty())),
        )

        is GemAssetDetailRow.Row -> AssetInfoUIModel.RowUIModel.Row(row.row)
    }

    private fun balance(item: GemAssetBalanceRow): AssetInfoUIModel.RowUIModel.Balance {
        val row = item.row
        return AssetInfoUIModel.RowUIModel.Balance(
            type = row.viewType(),
            url = (row as? GemBalanceRow.Reserved)?.url,
            model = ListItemModel(
                title = row.title().text(context),
                subtitle = item.value.text(context),
                info = InfoSheetEntity.PendingUnconfirmedBalanceInfo.takeIf { row is GemBalanceRow.PendingUnconfirmed },
            ),
        )
    }
}
