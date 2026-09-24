package com.gemwallet.android.features.settings.in_app_notifications.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.wallet.core.primitives.InAppNotification
import uniffi.gemstone.GemNotificationDestination
import uniffi.gemstone.GemNotificationIcon
import uniffi.gemstone.GemNotificationRow
import uniffi.gemstone.notificationRows

data class NotificationRowUIModel(val id: String, val createdAt: Long, val destination: GemNotificationDestination?, val model: ListItemModel)

internal fun List<InAppNotification>.uiModels(context: Context): List<NotificationRowUIModel> = zip(notificationRows(map { it.toGem() })) { notification, row ->
    notification.uiModel(row, context)
}

private fun InAppNotification.uiModel(row: GemNotificationRow, context: Context): NotificationRowUIModel = NotificationRowUIModel(
    id = item.id,
    createdAt = createdAt,
    destination = row.destination,
    model = ListItemModel(
        title = row.title,
        titleTag = if (row.isUnread) context.getString(R.string.assets_tags_new) else null,
        titleTagStyle = ListItemTextStyle.Primary,
        titleExtra = row.subtitle,
        subtitle = row.value,
        subtitleStyle = ListItemTextStyle.Body,
        subtitleExtra = row.subvalue,
        image = row.icon?.image(),
    ),
)

private fun GemNotificationIcon.image(): ListItemImage? = when (this) {
    is GemNotificationIcon.Emoji -> ListItemImage.Emoji(glyph)
    is GemNotificationIcon.Image -> ListItemImage.Url(url)
    is GemNotificationIcon.Asset -> ListItemImage.Asset(icon)
}
