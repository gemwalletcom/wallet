package com.gemwallet.android.features.bridge.views

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
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
import com.gemwallet.android.features.bridge.viewmodels.ProposalSceneState
import com.gemwallet.android.features.bridge.viewmodels.ProposalSceneViewModel
import com.gemwallet.android.features.bridge.viewmodels.model.SessionUI
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.WalletItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.components.color
import com.gemwallet.android.ui.components.icon
import com.gemwallet.android.ui.components.titleRes
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.pendingColor
import com.reown.walletkit.client.Wallet
import uniffi.gemstone.WalletConnectionVerificationStatus

@Composable
fun ProposalScene(
    proposal: Wallet.Model.SessionProposal,
    verifyContext: Wallet.Model.VerifyContext,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    val viewModel: ProposalSceneViewModel = hiltViewModel()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val peer by viewModel.proposal.collectAsStateWithLifecycle()
    val selectedWallet by viewModel.selectedWallet.collectAsStateWithLifecycle()
    val availableWallets by viewModel.availableWallets.collectAsStateWithLifecycle()

    DisposableEffect(key1 = proposal) {
        viewModel.onProposal(proposal, verifyContext)

        onDispose { viewModel.reset() }
    }

    when {
        state is ProposalSceneState.Canceled -> onCancel()
        state is ProposalSceneState.ScamCanceled -> {
            Toast.makeText(
                context,
                stringResource(R.string.errors_connections_malicious_origin),
                Toast.LENGTH_LONG
            ).show()
            onCancel()
        }
        state is ProposalSceneState.Fail -> FatalStateScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            message = (state as ProposalSceneState.Fail).message,
            onCancel = onCancel,
        )
        peer == null && state is ProposalSceneState.Init -> LoadingScene(
            title = stringResource(id = R.string.wallet_connect_connect_title),
            onCancel = onCancel,
            closeIcon = true,
        )
        else -> Proposal(
            peer = peer!!,
            verificationStatus = (state as ProposalSceneState.Init).verificationStatus,
            selectedWallet = selectedWallet,
            availableWallets = availableWallets,
            onReject = viewModel::onReject,
            onApprove = viewModel::onApprove,
            onWalletSelected = viewModel::onWalletSelected
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun Proposal(
    peer: SessionUI,
    verificationStatus: WalletConnectionVerificationStatus,
    selectedWallet: com.wallet.core.primitives.Wallet?,
    availableWallets: List<com.wallet.core.primitives.Wallet>,
    onReject: () -> Unit,
    onApprove: () -> Unit,
    onWalletSelected: (String) -> Unit,
) {
    var isShowSelectWallets by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(id = R.string.wallet_connect_connect_title),
        backHandle = true,
        closeIcon = true,
        mainAction = {
            MainActionButton(
                enabled = selectedWallet != null,
                title = stringResource(id = R.string.transfer_confirm),
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
                    icon = peer.icon,
                    title = peer.name,
                    subtitle = peer.uri,
                    contentDescription = "wallet_connect_app_icon",
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item {
                PropertyItem(
                    modifier = Modifier.clickable { isShowSelectWallets = true },
                    title = { PropertyTitleText(R.string.common_wallet) },
                    data = { PropertyDataText(selectedWallet?.name ?: "", badge = { DataBadgeChevron() }) },
                    listPosition = ListPosition.First,
                )
            }
            item {
                PropertyItem(
                    title = { PropertyTitleText(R.string.wallet_connect_connection_title) },
                    data = { PropertyDataText(stringResource(R.string.wallet_connect_brand_name)) },
                    listPosition = ListPosition.Middle,
                )
            }
            item {
                PropertyItem(
                    title = { PropertyTitleText(R.string.transaction_status) },
                    data = {
                        PropertyDataText(
                            text = stringResource(verificationStatus.titleRes()),
                            color = verificationStatus.color(),
                            badge = {
                                DataBadgeChevron(isShowChevron = false) {
                                    Icon(
                                        imageVector = verificationStatus.icon(),
                                        contentDescription = null,
                                        tint = verificationStatus.color(),
                                    )
                                }
                            },
                        )
                    },
                    listPosition = ListPosition.Last,
                )
            }
            item { SubheaderItem(R.string.wallet_connect_permissions_title) }
            permissionsContent()
        }
    }

    if (isShowSelectWallets) {
        ModalBottomSheet(
            dragHandle = { BottomSheetDefaults.DragHandle() },
            onDismissRequest = { isShowSelectWallets = false },
        ) {
            LazyColumn {
                itemsIndexed(availableWallets) { index, item ->
                    WalletItem(
                        wallet = item,
                        isCurrent = item.id == selectedWallet?.id,
                        listPosition = ListPosition.getPosition(index, availableWallets.size),
                        modifier = Modifier.clickable {
                            onWalletSelected(item.id)
                            isShowSelectWallets = false
                        }
                    )
                }
            }
        }
    }
}

private fun LazyListScope.permissionsContent() {
    val permissions = listOf(
        R.string.wallet_connect_permissions_view_balance,
        R.string.wallet_connect_permissions_approval_requests,
    )
    itemsIndexed(permissions) { index, item ->
        ListItem(
            listPosition = ListPosition.getPosition(index, permissions.size),
            minHeight = ListItemDefaults.plainMinHeight,
            leading = {
                Icon(
                    imageVector = Icons.Default.Check,
                    contentDescription = "",
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            },
            title = { PropertyTitleText(item) },
        )
    }
}
