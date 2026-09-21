package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.stake.viewmodels.localization.stringRes
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import uniffi.gemstone.GemStakeAction
import uniffi.gemstone.GemStakeActionItem
import uniffi.gemstone.GemStakeDestination

data class StakeActionUIModel(val destination: GemStakeDestination, val isEnabled: Boolean, val requiresFrozenBalance: Boolean, val model: ListItemModel)

internal fun GemStakeActionItem.uiModel(context: Context, assetInfo: AssetInfo): StakeActionUIModel = StakeActionUIModel(
    destination = destination,
    isEnabled = isEnabled,
    requiresFrozenBalance = requiresFrozenBalance,
    model = ListItemModel(
        title = context.getString(action.stringRes()),
        titleStyle = if (requiresFrozenBalance) ListItemTextStyle.Faded else ListItemTextStyle.Body,
        subtitle = value?.text(),
        info = InfoSheetEntity.StakeFrozenRequired(assetInfo.id().iconModel()).takeIf { requiresFrozenBalance },
    ),
)
