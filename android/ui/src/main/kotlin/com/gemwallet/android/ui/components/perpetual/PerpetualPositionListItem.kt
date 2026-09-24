package com.gemwallet.android.ui.components.perpetual

import android.content.Context
import com.gemwallet.android.domains.balance.hiddenWhen
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemPerpetualPositionRow

fun PerpetualPositionDataAggregate.listItem(context: Context, hideBalance: Boolean = false): ListItemModel = ListItemModel(
    title = title,
    titleExtra = positionLabel.string(context),
    titleExtraStyle = direction.textStyle(),
    subtitle = marginAmount.hiddenWhen(hideBalance),
    subtitleStyle = ListItemTextStyle.Body,
    subtitleExtra = pnl.string(context).hiddenWhen(hideBalance),
    subtitleExtraStyle = pnlState.textStyle(),
    image = ListItemImage.Asset(asset.id),
)

fun GemPerpetualPositionRow.listItem(context: Context, hideBalance: Boolean = false): ListItemModel = ListItemModel(
    title = title,
    titleExtra = position.string(context),
    titleExtraStyle = directionTone.textStyle(),
    subtitle = margin.text().hiddenWhen(hideBalance),
    subtitleStyle = ListItemTextStyle.Body,
    subtitleExtra = pnl.string(context).hiddenWhen(hideBalance),
    subtitleExtraStyle = pnlTone.textStyle(),
    image = ListItemImage.Asset(icon),
)
