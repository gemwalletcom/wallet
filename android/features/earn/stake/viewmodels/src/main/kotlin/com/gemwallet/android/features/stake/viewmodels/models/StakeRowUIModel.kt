package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.stake.viewmodels.localization.stringRes
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import uniffi.gemstone.GemStakeActionItem
import uniffi.gemstone.GemStakeActionTap

data class StakeActionUIModel(val tap: GemStakeActionTap, val model: ListItemModel)

internal fun GemStakeActionItem.uiModel(context: Context, assetInfo: AssetInfo): StakeActionUIModel {
    val needsFrozenBalance = tap is GemStakeActionTap.FrozenBalanceInfo
    return StakeActionUIModel(
        tap = tap,
        model = ListItemModel(
            title = context.getString(action.stringRes()),
            titleStyle = if (needsFrozenBalance) ListItemTextStyle.Faded else ListItemTextStyle.Body,
            subtitle = value?.text(),
            info = InfoSheetEntity.StakeFrozenRequired(assetInfo.id().iconModel()).takeIf { needsFrozenBalance },
        ),
    )
}
