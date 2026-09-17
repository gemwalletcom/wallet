package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetAction
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireOptionUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset

@Composable
internal fun GetAssetBottomSheet(
    asset: Asset?,
    options: List<AcquireOptionUIModel>,
    onDismiss: () -> Unit,
    onAction: (AcquireAssetAction) -> Unit,
) {
    ModalBottomSheet(
        item = asset,
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = { stringResource(R.string.asset_get_asset, it.symbol) },
    ) {
        Column(modifier = Modifier.padding(bottom = paddingDefault)) {
            options.forEachIndexed { index, option ->
                ListItem(
                    model = option.model,
                    listPosition = ListPosition.getPosition(index, options.size),
                    modifier = Modifier.clickable { onAction(option.action) },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
