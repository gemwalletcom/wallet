@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.bridge.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.WalletSectionUIModel
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.walletSections
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemSimulationPayloadRow

internal fun LazyListScope.walletConnectTextMessage(message: String) {
    item {
        SubheaderItem(R.string.sign_message_message)
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .listItem()
                .padding(paddingDefault),
            text = message,
        )
    }
}

@Composable
internal fun WalletConnectPayloadDetailsSheet(
    isVisible: Boolean,
    primaryFields: List<GemSimulationPayloadRow>,
    secondaryFields: List<GemSimulationPayloadRow>,
    onAddressClick: (String) -> Unit,
    onViewFullMessage: () -> Unit,
    onDismissRequest: () -> Unit,
    viewFullMessageListItem: ListItemModel,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        expansion = SheetExpansion.Full,
        onDismissRequest = onDismissRequest,
        title = stringResource(R.string.common_details),
    ) {
        LazyColumn {
            simulationPayloadDetailsContent(
                primaryFields = primaryFields,
                secondaryFields = secondaryFields,
                onAddressClick = onAddressClick,
            )
            item {
                ListItem(
                    model = viewFullMessageListItem,
                    listPosition = ListPosition.Single,
                    modifier = Modifier.clickable(onClick = onViewFullMessage),
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}

@Composable
internal fun WalletConnectFullMessageSheet(isVisible: Boolean, message: String, onDismissRequest: () -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        expansion = SheetExpansion.Full,
        onDismissRequest = onDismissRequest,
        title = stringResource(R.string.sign_message_view_full_message),
    ) {
        LazyColumn(
            contentPadding = PaddingValues(paddingDefault),
        ) {
            item {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = message,
                )
            }
        }
    }
}

@Composable
internal fun WalletSelectionSheet(isVisible: Boolean, walletSections: List<WalletSectionUIModel>, selectedWalletId: WalletId?, onWalletSelected: (WalletId) -> Unit, onDismissRequest: () -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        dragHandle = { BottomSheetDefaults.DragHandle() },
        onDismissRequest = onDismissRequest,
    ) {
        LazyColumn {
            item { SubheaderItem(R.string.wallets_title) }
            walletSections(walletSections, selectedWalletId?.id) { id ->
                onWalletSelected(WalletId(id))
                onDismissRequest()
            }
        }
    }
}
