package com.gemwallet.android.features.transfer.presents.receive

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemReceiveNetwork

@Composable
internal fun ReceiveNetworkSelector(isVisible: Boolean, networks: List<GemReceiveNetwork>, onSelect: (AssetId) -> Unit, onDismiss: () -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = stringResource(R.string.settings_networks_title),
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = paddingDefault),
        ) {
            itemsIndexed(networks) { index, network ->
                val assetId = network.assetId.toAssetId()!!
                ChainItem(
                    row = network.row,
                    listPosition = ListPosition.getPosition(index, networks.size),
                    trailing = { DataBadgeChevron() },
                    onClick = {
                        onSelect(assetId)
                        onDismiss()
                    },
                )
            }
        }
    }
}
