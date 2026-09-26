package com.gemwallet.android.features.assets.viewmodels.details.models

import android.content.Context
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.banner.uiModel
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemAssetBalanceRow
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemInfoTopic
import javax.inject.Inject

class AssetInfoUIModelFactory @Inject constructor(@ApplicationContext private val context: Context) {

    fun create(chainAssetInfo: ChainAssetInfo, details: GemAssetDetails): AssetInfoUIModel {
        val assetInfo = chainAssetInfo.assetInfo
        val asset = assetInfo.asset
        return AssetInfoUIModel(
            assetInfo = assetInfo,
            details = details,
            priceAlertMenu = details.state.priceAlert.menu(),
            emptyTransactions = details.state.emptyTransactionsAction.emptyTransactions(),
            banner = details.banner?.uiModel(context),
            sections = details.sections.map { section ->
                AssetInfoUIModel.SectionUIModel(section.title.titleRes(), section.rows.map { row(it, asset.id, details.networkDestination.navigation()) })
            },
        )
    }

    private fun row(row: GemAssetDetailRow, assetId: AssetId, network: AssetDetailsAction.Navigation?): AssetInfoUIModel.RowUIModel = when (row) {
        is GemAssetDetailRow.Balance -> balance(row.row, row.tap?.detailsAction(assetId, network))
        is GemAssetDetailRow.Row -> AssetInfoUIModel.RowUIModel.Row(row.row, row.tap?.detailsAction(assetId, network))
    }

    private fun balance(item: GemAssetBalanceRow, action: AssetDetailsAction?): AssetInfoUIModel.RowUIModel.Balance {
        val row = item.row
        return AssetInfoUIModel.RowUIModel.Balance(
            model = ListItemModel(
                title = row.title().text(context),
                subtitle = item.value.text(context),
                info = GemInfoTopic.PendingUnconfirmedBalance.infoSheet().takeIf { row is GemBalanceRow.PendingUnconfirmed },
            ),
            action = action,
        )
    }
}
