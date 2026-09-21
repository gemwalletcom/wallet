package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import uniffi.gemstone.GemConnection

data class ConnectionRowUIModel(val id: String, val model: ListItemModel)

internal fun GemConnection.listItem(): ListItemModel {
    val placeholder = row.initial ?: "WC"
    return ListItemModel(
        title = row.title,
        titleExtra = row.host,
        image = row.iconUrl?.let { ListItemImage.Url(it, placeholder = placeholder) } ?: ListItemImage.Initials(placeholder),
    )
}

internal fun GemConnection.rowUIModel(): ConnectionRowUIModel = ConnectionRowUIModel(id = connection.session.id, model = listItem())
