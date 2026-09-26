package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.iconResource
import com.gemwallet.android.ui.localization.actionRes
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemFiatTransactionRow

fun GemFiatTransactionRow.listItemModel(context: Context): ListItemModel = ListItemModel(
    title = context.getString(quoteType.toPrimitives().actionRes()),
    titleTag = badge?.let { context.getString(it.stringRes()) },
    titleTagStyle = badge?.textStyle() ?: ListItemTextStyle.Secondary,
    titleExtra = subtitle,
    subtitle = value.text(),
    subtitleStyle = value.tone.textStyle(),
    subtitleExtra = fiatValue.text(),
    image = ListItemImage.Drawable(provider.toPrimitives().iconResource()),
)
