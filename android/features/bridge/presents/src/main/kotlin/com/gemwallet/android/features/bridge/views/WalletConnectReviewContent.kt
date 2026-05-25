@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.bridge.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.WalletItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

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
    primaryFields: List<PayloadField>,
    secondaryFields: List<PayloadField>,
    onViewFullMessage: () -> Unit,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        onDismissRequest = onDismissRequest,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        title = stringResource(R.string.common_details),
    ) {
        LazyColumn {
            simulationPayloadDetailsContent(
                primaryFields = primaryFields,
                secondaryFields = secondaryFields,
            )
            item {
                PropertyItem(
                    action = R.string.sign_message_view_full_message,
                    listPosition = ListPosition.Single,
                    onClick = onViewFullMessage,
                )
            }
        }
    }
}

@Composable
internal fun WalletConnectFullMessageSheet(
    message: String,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        onDismissRequest = onDismissRequest,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
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
internal fun WalletSelectionSheet(
    isVisible: Boolean,
    wallets: List<Wallet>,
    selectedWalletId: WalletId?,
    onWalletSelected: (WalletId) -> Unit,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        dragHandle = { BottomSheetDefaults.DragHandle() },
        onDismissRequest = onDismissRequest,
    ) {
        LazyColumn {
            item { SubheaderItem(R.string.wallets_title) }
            itemsIndexed(wallets) { index, wallet ->
                WalletItem(
                    wallet = wallet,
                    isCurrent = wallet.id == selectedWalletId,
                    listPosition = ListPosition.getPosition(index, wallets.size),
                    modifier = Modifier.clickable {
                        onWalletSelected(wallet.id)
                        onDismissRequest()
                    },
                )
            }
        }
    }
}
