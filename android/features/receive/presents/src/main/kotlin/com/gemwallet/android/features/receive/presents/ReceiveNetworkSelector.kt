package com.gemwallet.android.features.receive.presents

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId

@Composable
internal fun ReceiveNetworkSelector(
    isVisible: Boolean,
    assetIds: List<AssetId>,
    onSelect: (AssetId) -> Unit,
    onDismiss: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.settings_networks_title),
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = paddingDefault),
        ) {
            itemsIndexed(assetIds) { index, assetId ->
                ChainItem(
                    title = assetId.chain.networkName(),
                    icon = assetId.chain,
                    subtitle = assetId.chain.assetType()?.string,
                    listPosition = ListPosition.getPosition(index, assetIds.size),
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
