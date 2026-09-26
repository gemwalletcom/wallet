package com.gemwallet.android.features.wallet_tab.viewmodels.models

import com.gemwallet.android.domains.asset.getListIconUrl
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.AssetList

data class AssetListRowUIModel(val id: String, val name: String, val model: ListItemModel)

internal fun AssetList.uiModel(): AssetListRowUIModel = AssetListRowUIModel(
    id = id,
    name = name,
    model = ListItemModel(title = name, subtitle = count.toString(), image = ListItemImage.Url(getListIconUrl(id), placeholder = name)),
)
