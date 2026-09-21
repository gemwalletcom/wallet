@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.referral.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.features.referral.viewmodels.ReferralViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.WalletItem
import com.gemwallet.android.ui.components.list_item.uiModel
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import kotlinx.coroutines.launch

@Composable
fun ReferralNavScreen(onClose: () -> Unit, viewModel: ReferralViewModel = hiltViewModel()) {
    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val loadingMessage = stringResource(R.string.common_loading)
    val insufficientPointsMessage = stringResource(R.string.rewards_insufficient_points)
    val doneMessage = stringResource(R.string.common_done)

    var isShowSelectWallets by remember { mutableStateOf(false) }
    var showErrorDialog by remember { mutableStateOf<Throwable?>(null) }
    var showMessageDialog by remember { mutableStateOf<String?>(null) }

    val availableWallets by viewModel.availableWallets.collectAsStateWithLifecycle()
    val availableWalletRows by viewModel.availableWalletRows.collectAsStateWithLifecycle()
    val currentWallet by viewModel.currentWallet.collectAsStateWithLifecycle()
    val referralLink by viewModel.referralLink.collectAsStateWithLifecycle()
    val inSync by viewModel.inSync.collectAsStateWithLifecycle()
    val incomingCode by viewModel.incomingCode.collectAsStateWithLifecycle()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val infoRows by viewModel.infoRows.collectAsStateWithLifecycle()
    val redemptions by viewModel.redemptions.collectAsStateWithLifecycle()

    ReferralScene(
        inSync = inSync,
        isAvailableWalletSelect = availableWallets.size > 1,
        incomingCode = incomingCode,
        referralLink = referralLink,
        uiState = uiState,
        infoRows = infoRows,
        redemptions = redemptions,
        currentWallet = currentWallet,
        onUsername = viewModel::createReferral,
        onCode = viewModel::useCode,
        onCancelCode = viewModel::cancelCode,
        onRefresh = viewModel::sync,
        onWallet = { isShowSelectWallets = true },
        onRedeem = {
            if (!it.redemption.canRedeem) {
                showMessageDialog = insufficientPointsMessage
                return@ReferralScene
            }
            scope.launch { snackbar.showSnackbar(loadingMessage, R.drawable.ic_refresh) }
            viewModel.redeem(it.redemption) { err ->
                if (err == null) {
                    scope.launch { snackbar.showSnackbar(doneMessage, R.drawable.ic_check_circle) }
                } else {
                    showErrorDialog = err
                }
            }
        },
        onClose = onClose,
        snackbar = snackbar,
    )

    ModalBottomSheet(
        isVisible = isShowSelectWallets,
        onDismissRequest = { isShowSelectWallets = false },
        title = stringResource(R.string.wallets_title),
    ) {
        LazyColumn {
            itemsIndexed(availableWalletRows) { index, item ->
                WalletItem(
                    model = item.uiModel(LocalContext.current),
                    isCurrent = item.id == currentWallet?.id?.id,
                    listPosition = ListPosition.getPosition(index, availableWalletRows.size),
                    modifier = Modifier.clickable {
                        viewModel.setWallet(walletId = item.id)
                        isShowSelectWallets = false
                    },
                )
            }
        }
    }

    if (showMessageDialog != null) {
        AlertDialog(
            containerColor = MaterialTheme.colorScheme.background,
            onDismissRequest = { showMessageDialog = null },
            text = { Text(showMessageDialog.orEmpty()) },
            confirmButton = {
                Button({ showMessageDialog = null }) { Text(stringResource(R.string.common_cancel)) }
            },
        )
    }

    if (showErrorDialog != null) {
        val message = showErrorDialog?.errorText()?.text() ?: stringResource(R.string.transaction_status_failed)
        AlertDialog(
            containerColor = MaterialTheme.colorScheme.background,
            onDismissRequest = { showErrorDialog = null },
            text = { Text(message) },
            confirmButton = {
                Button({ showErrorDialog = null }) { Text(stringResource(R.string.common_cancel)) }
            },
        )
    }
}
