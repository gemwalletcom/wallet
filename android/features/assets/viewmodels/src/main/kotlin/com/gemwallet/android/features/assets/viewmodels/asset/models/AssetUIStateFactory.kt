package com.gemwallet.android.features.assets.viewmodels.asset.models

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.banner.uiModel
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAssetData
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemAssetBalanceRow
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemInfoTopic
import javax.inject.Inject

class AssetUIStateFactory @Inject constructor(@ApplicationContext private val context: Context) {

    fun create(chainAssetInfo: ChainAssetData, details: GemAssetDetails): AssetUIState {
        val assetInfo = chainAssetInfo.assetData
        val asset = assetInfo.asset
        return AssetUIState(
            assetInfo = assetInfo,
            details = details,
            priceAlertMenu = details.state.priceAlert.menu(),
            banner = details.banner?.uiModel(context),
            sections = details.sections.map { section ->
                AssetDetailSectionUIModel(section.title.titleRes(), section.rows.map { row(it, asset.id, details.networkDestination.navigation()) })
            },
        )
    }

    private fun row(row: GemAssetDetailRow, assetId: AssetId, network: AssetAction.Navigation?): AssetDetailRowUIModel = when (row) {
        is GemAssetDetailRow.Balance -> balance(row.row, row.action?.detailsAction(assetId, network))
        is GemAssetDetailRow.Row -> AssetDetailRowUIModel.Row(row.row, row.action?.detailsAction(assetId, network))
    }

    private fun balance(item: GemAssetBalanceRow, action: AssetAction?): AssetDetailRowUIModel.Balance {
        val row = item.row
        return AssetDetailRowUIModel.Balance(
            model = ListItemModel(
                title = row.title().text(context),
                subtitle = item.value.text(context),
                info = GemInfoTopic.PendingUnconfirmedBalance.infoSheet().takeIf { row is GemBalanceRow.PendingUnconfirmed },
            ),
            action = action,
        )
    }
}
