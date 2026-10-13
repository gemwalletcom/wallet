@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.rewards.presents

import androidx.compose.foundation.lazy.LazyColumn
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
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.rewards.viewmodels.RewardsViewModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.walletSections
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.shareText

@Composable
fun RewardsScreen(onClose: () -> Unit, viewModel: RewardsViewModel = hiltViewModel()) {
    val infoSheet by viewModel.infoSheet.collectAsStateWithLifecycle()
    InfoBottomSheet(item = infoSheet?.infoSheet(), onClose = { viewModel.infoSheet.value = null })
    val snackbar = remember { SnackbarHostState() }
    ToastEffect(viewModel.toastEvents, snackbar)
    val context = LocalContext.current

    var isShowSelectWallets by remember { mutableStateOf(false) }

    val error by viewModel.error.collectAsStateWithLifecycle()
    val sheet by viewModel.sheet.collectAsStateWithLifecycle()
    val isLoading by viewModel.isLoading.collectAsStateWithLifecycle()
    val wallet by viewModel.wallet.collectAsStateWithLifecycle()
    val referralLink by viewModel.referralLink.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val incomingCode by viewModel.incomingCode.collectAsStateWithLifecycle()
    val introItems by viewModel.introItems.collectAsStateWithLifecycle()
    val inviteAction by viewModel.inviteAction.collectAsStateWithLifecycle()
    val canUseReferralCode by viewModel.canUseReferralCode.collectAsStateWithLifecycle()
    val pendingReferral by viewModel.pendingReferral.collectAsStateWithLifecycle()
    val notices by viewModel.notices.collectAsStateWithLifecycle()
    val inviteDescription by viewModel.inviteDescription.collectAsStateWithLifecycle()
    val shareText by viewModel.shareText.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val redemptions by viewModel.redemptions.collectAsStateWithLifecycle()

    val shareTitle = stringResource(R.string.common_share, referralLink.orEmpty())

    RewardsScene(
        state = state,
        isRefreshing = isRefreshing,
        wallet = wallet,
        incomingCode = incomingCode,
        introItems = introItems,
        inviteAction = inviteAction,
        canUseReferralCode = canUseReferralCode,
        pendingReferral = pendingReferral,
        notices = notices,
        inviteDescription = inviteDescription,
        sections = sections,
        redemptions = redemptions,
        sheet = sheet,
        isLoading = isLoading,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                RewardsAction.Refresh -> viewModel.sync()

                RewardsAction.Close -> onClose()

                RewardsAction.SelectWallet -> isShowSelectWallets = true

                RewardsAction.InviteFriends -> viewModel.inviteFriends {
                    context.shareText(subject = referralLink.orEmpty(), text = shareText.orEmpty(), chooserTitle = shareTitle)
                }

                RewardsAction.CodeHandled -> viewModel.onCodeHandled()

                RewardsAction.DismissSheet -> viewModel.dismissSheet()

                is RewardsAction.ShowSheet -> viewModel.showSheet(action.sheet)

                is RewardsAction.CreateReferral -> viewModel.createReferral(action.username)

                is RewardsAction.UseCode -> viewModel.useCode(action.code)

                is RewardsAction.Redeem -> viewModel.onSelectRedemption(action.redemption) {
                    context.requestAuth(AuthRequest.Default) { viewModel.redeem(action.redemption.id) }
                }
            }
        },
    )

    ModalBottomSheet(
        isVisible = isShowSelectWallets,
        onDismissRequest = { isShowSelectWallets = false },
        title = stringResource(R.string.wallets_title),
    ) {
        val onSelectWallet: (String) -> Unit = { id ->
            viewModel.setWallet(walletId = id)
            isShowSelectWallets = false
        }
        LazyColumn {
            walletSections(wallet?.sections.orEmpty(), wallet?.row?.id, onSelectWallet)
        }
    }

    error?.let { message ->
        AlertDialog(
            containerColor = MaterialTheme.colorScheme.background,
            onDismissRequest = viewModel::clearError,
            text = { Text(message) },
            confirmButton = {
                Button(viewModel::clearError) { Text(stringResource(R.string.common_cancel)) }
            },
        )
    }
}
