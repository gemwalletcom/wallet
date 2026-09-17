package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition

fun LazyListScope.manageAssetItem(
    uiState: AssetInfoUIModel,
    onPin: () -> Unit,
    onAdd: () -> Unit,
) {
    if (!uiState.detailsState.showsManage) {
        return
    }

    item {
        ListItem(
            model = uiState.pinListItem,
            listPosition = ListPosition.First,
            modifier = Modifier.clickable(onClick = onPin),
            accessory = { DataBadgeChevron() },
        )
        ListItem(
            model = uiState.addListItem,
            listPosition = ListPosition.Last,
            modifier = Modifier.clickable(onClick = onAdd),
            accessory = { DataBadgeChevron() },
        )
    }
}
