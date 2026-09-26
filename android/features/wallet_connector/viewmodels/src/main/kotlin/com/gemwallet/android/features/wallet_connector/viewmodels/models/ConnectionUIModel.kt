package com.gemwallet.android.features.wallet_connector.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import uniffi.gemstone.GemConnection

data class ConnectionUIModel(val id: String, val model: ListItemModel)

internal fun GemConnection.listItem(): ListItemModel {
    val placeholder = row.initial
    return ListItemModel(
        title = row.title,
        titleExtra = row.host,
        image = row.iconUrl?.let { ListItemImage.Url(it, placeholder = placeholder) } ?: ListItemImage.Initials(placeholder),
    )
}

internal fun GemConnection.uiModel(): ConnectionUIModel = ConnectionUIModel(id = connection.session.id, model = listItem())
