package com.gemwallet.android.features.wallet.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun ShowSecretDataProperty(
    label: String,
    onClick: () -> Unit,
) {
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = {
            PropertyTitleText(stringResource(R.string.common_show, label))
        },
        data = { PropertyDataText("", badge = { DataBadgeChevron() }) },
        listPosition = ListPosition.Single,
    )
}
