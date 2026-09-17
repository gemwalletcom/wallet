package com.gemwallet.android.features.bridge.views

import com.gemwallet.android.ui.components.list_item.ListItemModel
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.features.bridge.viewmodels.ProposalSceneState
import com.gemwallet.android.features.bridge.viewmodels.ProposalSceneViewModel
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.style.icon
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.WalletConnectionVerificationStatus

@Composable
fun ProposalScene(
    proposal: WalletConnectSessionProposal,
    verifyContext: WalletConnectVerifyContext,
    onError: (String) -> Unit,
) {
    val context = LocalContext.current
    val viewModel: ProposalSceneViewModel = hiltViewModel()
    BackHandler(onBack = viewModel::onReject)
    val state by viewModel.state.collectAsStateWithLifecycle()
    val walletListItem by viewModel.walletListItem.collectAsStateWithLifecycle()
    val statusListItem by viewModel.statusListItem.collectAsStateWithLifecycle()
    val peer by viewModel.proposal.collectAsStateWithLifecycle()
    val selectedWallet by viewModel.selectedWallet.collectAsStateWithLifecycle()
    val availableWallets by viewModel.availableWallets.collectAsStateWithLifecycle()
    val availableWalletRows by viewModel.availableWalletRows.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val unknownErrorMessage = stringResource(id = R.string.errors_unknown_try_again)

    LaunchedEffect(proposal) {
        viewModel.onProposal(proposal, verifyContext) { error ->
            when (error) {
                BridgeRequestError.MaliciousSession -> Toast.makeText(
                    context,
                    R.string.errors_connections_malicious_origin,
                    Toast.LENGTH_LONG
                ).show()
                BridgeRequestError.Expired -> Toast.makeText(
                    context,
                    R.string.wallet_connect_request_expired,
                    Toast.LENGTH_LONG
                ).show()
            }
        }
    }

    when (val currentPeer = peer) {
        null -> LoadingScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            onCancel = viewModel::onReject,
            closeIcon = true,
        )
        else -> Proposal(
            peer = currentPeer,
            state = state,
            walletListItem = walletListItem,
            connectionListItem = viewModel.connectionListItem,
            statusListItem = statusListItem,
            permissionListItems = viewModel.permissionListItems,
            selectedWallet = selectedWallet,
            availableWallets = availableWallets,
            availableWalletRows = availableWalletRows,
            buttonState = buttonState,
            onReject = viewModel::onReject,
            onApprove = { viewModel.onApprove { error -> onError(error.text(context).ifBlank { unknownErrorMessage }) } },
            onWalletSelected = viewModel::onWalletSelected
        )
    }
}

@Composable
private fun Proposal(
    peer: GemConnectionRow,
    state: ProposalSceneState,
    walletListItem: ListItemModel,
    connectionListItem: ListItemModel,
    statusListItem: ListItemModel,
    permissionListItems: List<ListItemModel>,
    selectedWallet: com.wallet.core.primitives.Wallet?,
    availableWallets: List<com.wallet.core.primitives.Wallet>,
    availableWalletRows: List<uniffi.gemstone.GemWalletRow>,
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
                onClick = onApprove
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
                    modifier = if (state is ProposalSceneState.Approving) {
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

    WalletSelectionSheet(
        isVisible = isShowSelectWallets,
        walletRows = availableWalletRows,
        selectedWalletId = selectedWallet?.id,
        onWalletSelected = onWalletSelected,
        onDismissRequest = { isShowSelectWallets = false },
    )
}

