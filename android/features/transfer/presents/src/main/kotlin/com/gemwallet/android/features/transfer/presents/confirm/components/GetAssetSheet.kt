package com.gemwallet.android.features.transfer.presents.confirm.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.localization.subtitle
import com.gemwallet.android.ui.localization.title
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.image
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAcquireOption

@Composable
internal fun GetAssetSheet(asset: Asset?, options: List<GemAcquireOption>, onDismiss: () -> Unit, onSelect: (GemAcquireOption) -> Unit) {
    val context = LocalContext.current
    ModalBottomSheet(
        item = asset,
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = { stringResource(R.string.asset_get_asset, it.symbol) },
    ) {
        Column(modifier = Modifier.padding(bottom = paddingDefault)) {
            options.forEachIndexed { index, option ->
                ListItem(
                    model = ListItemModel(title = option.title(context), titleExtra = option.subtitle(context), image = option.image()),
                    listPosition = ListPosition.getPosition(index, options.size),
                    modifier = Modifier.clickable { onSelect(option) },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
