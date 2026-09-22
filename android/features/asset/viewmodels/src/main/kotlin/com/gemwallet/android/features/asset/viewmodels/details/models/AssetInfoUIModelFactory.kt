package com.gemwallet.android.features.asset.viewmodels.details.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.banner.uiModel
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemAssetBalanceRow
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemBannerRow
import javax.inject.Inject

class AssetInfoUIModelFactory @Inject constructor(@ApplicationContext private val context: Context) {

    fun create(chainAssetInfo: ChainAssetInfo, details: GemAssetDetails, banners: List<GemBannerRow>): AssetInfoUIModel {
        val assetInfo = chainAssetInfo.assetInfo
        val asset = assetInfo.asset
        return AssetInfoUIModel(
            assetInfo = assetInfo,
            name = details.title,
            iconUrl = asset.id.iconModel(),
            tokenType = asset.type,
            isBuyEnabled = assetInfo.metadata.isBuyEnabled,
            isSwapEnabled = assetInfo.metadata.isSwapEnabled,
            swapPayAssetId = details.swapPair.payAssetId.toAssetId(),
            swapReceiveAssetId = details.swapPair.receiveAssetId?.toAssetId(),
            explorerName = details.explorerName,
            explorerAddressUrl = details.addressLink?.link,
            explorerTokenUrl = details.tokenLink?.link,
            verificationStatus = details.verificationStatus?.toPrimitives(),
            networkAction = details.networkDestination.navigation(),
            shareUrl = details.shareUrl,
            detailsState = details.state,
            priceAlertMenu = details.state.priceAlert.menu(),
            emptyTransactions = details.state.emptyTransactionsAction.emptyTransactions(),
            banners = banners.map { it.uiModel(context) },
            sections = details.sections.map { section ->
                AssetInfoUIModel.SectionUIModel(section.title.titleRes(), section.rows.map { row(it, asset.id, details.networkDestination.navigation()) })
            },
            accountInfoUIModel = AssetInfoUIModel.AccountInfoUIModel(
                totalBalance = details.balanceValue.text(),
                totalFiat = details.fiatValue?.text().orEmpty(),
                owner = assetInfo.owner?.address ?: "",
            ),
        )
    }

    private fun row(row: GemAssetDetailRow, assetId: AssetId, network: AssetDetailsAction.Navigation?): AssetInfoUIModel.RowUIModel = when (row) {
        is GemAssetDetailRow.Balance -> balance(row.row)
        is GemAssetDetailRow.Row -> AssetInfoUIModel.RowUIModel.Row(row.row, row.row.detailsAction(assetId) ?: row.row.networkAction(network))
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
