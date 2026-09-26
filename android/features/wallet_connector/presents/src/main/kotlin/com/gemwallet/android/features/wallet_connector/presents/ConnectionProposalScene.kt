package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.wallet_connector.viewmodels.ConnectionProposalUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.WalletSectionUIModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.style.icon
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemConnectionRow

@Composable
internal fun ConnectionProposalScene(
    peer: GemConnectionRow,
    state: ConnectionProposalUIState,
    walletListItem: ListItemModel,
    connectionListItem: ListItemModel,
    statusListItem: ListItemModel,
    permissionListItems: List<ListItemModel>,
    selectedWallet: com.wallet.core.primitives.Wallet?,
    availableWallets: List<com.wallet.core.primitives.Wallet>,
    availableWalletSections: List<WalletSectionUIModel>,
    buttonState: ButtonState,
    onReject: () -> Unit,
    onApprove: () -> Unit,
    onWalletSelected: (WalletId) -> Unit,
) {
    var isShowSelectWallets by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(id = R.string.wallet_connect_connect_title),
        backHandle = true,
        closeIcon = true,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.transfer_confirm),
                state = buttonState,
                onClick = onApprove,
            )
        },
        onClose = onReject,
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding() + paddingDefault),
        ) {
            item {
                CenteredListHead(
                    icon = peer.iconUrl,
                    title = peer.title,
                    subtitle = peer.host.orEmpty(),
                    contentDescription = "wallet_connect_app_icon",
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item {
                ListItem(
                    model = walletListItem,
                    listPosition = ListPosition.First,
                    modifier = if (state is ConnectionProposalUIState.Approving) {
                        Modifier
                    } else {
                        Modifier.clickable { isShowSelectWallets = true }
                    },
                    accessory = { DataBadgeChevron() },
                )
            }
            item { ListItem(model = connectionListItem, listPosition = ListPosition.Middle) }
            item {
                ListItem(
                    model = statusListItem,
                    listPosition = ListPosition.Last,
                    accessory = {
                        Icon(
                            imageVector = state.verificationStatus.icon(),
                            contentDescription = null,
                            tint = state.verificationStatus.color(),
                        )
                    },
                )
            }
            item { SubheaderItem(R.string.wallet_connect_permissions_title) }
            itemsPositioned(permissionListItems) { position, item ->
                ListItem(model = item, listPosition = position, minHeight = ListItemDefaults.plainMinHeight)
            }
        }
    }

    SelectWalletSheet(
        isVisible = isShowSelectWallets,
        walletSections = availableWalletSections,
        selectedWalletId = selectedWallet?.id,
        onWalletSelected = onWalletSelected,
        onDismissRequest = { isShowSelectWallets = false },
    )
}
