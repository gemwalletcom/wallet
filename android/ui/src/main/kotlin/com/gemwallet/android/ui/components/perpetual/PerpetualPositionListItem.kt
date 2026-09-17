package com.gemwallet.android.ui.components.perpetual

import android.content.Context
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.PerpetualProvider

fun PerpetualPositionDataAggregate.listItem(context: Context): ListItemModel = ListItemModel(
    title = title,
    titleExtra = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.positionText(context.getString(direction.stringRes()), leverage) },
    titleExtraStyle = direction.textStyle(),
    subtitle = marginAmount,
    subtitleStyle = ListItemTextStyle.Body,
    subtitleExtra = pnlWithPercentage,
    subtitleExtraStyle = pnlState.textStyle(),
    image = ListItemImage.Asset(asset.id),
)
