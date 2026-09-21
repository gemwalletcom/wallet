package com.gemwallet.android.features.referral.views

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.features.referral.viewmodels.SyncType
import com.gemwallet.android.features.referral.viewmodels.models.RewardRedemptionUIModel
import com.gemwallet.android.features.referral.views.components.referralHead
import com.gemwallet.android.features.referral.views.components.referralInfo
import com.gemwallet.android.features.referral.views.dialogs.GetStartedDialog
import com.gemwallet.android.features.referral.views.dialogs.ReferralCodeDialog
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.mainActionButtonColors
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.launch
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemRewardsState

private val referralCodeMaxWidth = 250.dp

@Composable
fun ReferralScene(
    inSync: SyncType,
    isAvailableWalletSelect: Boolean,
    referralLink: String?,
    uiState: GemRewardsState,
    infoRows: List<ListItemModel>,
    redemptions: List<RewardRedemptionUIModel>,
    currentWallet: Wallet?,
    incomingCode: GemIncomingCode? = null,
    onUsername: (String, (Exception?) -> Unit) -> Unit,
    onCode: (String, (Exception?) -> Unit) -> Unit,
    onCancelCode: () -> Unit,
    onRefresh: () -> Unit,
    onWallet: () -> Unit,
    onRedeem: (RewardRedemptionUIModel) -> Unit,
    onClose: () -> Unit,
    snackbar: SnackbarHostState = remember { SnackbarHostState() },
) {
    val context = LocalContext.current
    val link = referralLink.orEmpty()
    val joinText = stringResource(R.string.rewards_share_text, link)
    val shareTitle = stringResource(id = R.string.common_share, link)

    var getStartedDialogShow by remember(uiState) { mutableStateOf(false) }
    var codeDialogShow by remember(incomingCode, inSync) { mutableStateOf(incomingCode is GemIncomingCode.Confirm && inSync == SyncType.None) }
    val referralCode = (incomingCode as? GemIncomingCode.Confirm)?.code

    val successStr = stringResource(R.string.common_done)
    val scope = rememberCoroutineScope()

    val onShare = fun () {
        context.shareText(subject = link, text = joinText, chooserTitle = shareTitle)
    }

    val onCodeResult = fun (error: Exception?) {
        val message = error?.errorText()?.text(context)
        scope.launch {
            if (message == null) {
                snackbar.showSnackbar(successStr, R.drawable.ic_check_circle)
            } else {
                snackbar.showSnackbar(message, R.drawable.ic_error)
            }
        }
    }

    LaunchedEffect(incomingCode) {
        val code = (incomingCode as? GemIncomingCode.Activate)?.code ?: return@LaunchedEffect
        onCancelCode()
        onCode(code, onCodeResult)
    }

    Scene(
        title = stringResource(R.string.rewards_title),
        snackbar = snackbar,
        actions = {
            if (isAvailableWalletSelect) {
                Row(
                    modifier = Modifier
                        .widthIn(max = referralCodeMaxWidth)
                        .padding(horizontal = sceneContentPadding())
                        .clip(RoundedCornerShape(paddingDefault))
                        .background(MaterialTheme.colorScheme.primary, RoundedCornerShape(paddingDefault))
                        .clickable(onWallet)
                        .padding(start = paddingDefault, end = paddingSmall)
                        .padding(vertical = paddingSmall),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = currentWallet?.name ?: "",
                        maxLines = 1,
                        overflow = TextOverflow.MiddleEllipsis,
                        color = MaterialTheme.colorScheme.onPrimary,
                    )
                    Icon(
                        imageVector = AppIcons.KeyboardArrowDown,
                        contentDescription = "select wallet",
                        tint = MaterialTheme.colorScheme.onPrimary,
                    )
                }
            }
        },
        onClose = onClose,
    ) {
        if (inSync == SyncType.Init) {
            Box(modifier = Modifier.fillMaxSize()) {
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            }
            return@Scene
        }
        PullToRefreshBox(
            isRefreshing = inSync != SyncType.None,
            onRefresh = onRefresh,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                referralHead(
                    joinPointsCost = uiState.inviteRewardPoints.text(),
                    canInvite = uiState.canInvite,
                    hasCode = uiState.hasReferralCode,
                    onGetStarted = { getStartedDialogShow = true },
                    onShare = onShare,
                )

                if (uiState.canUseReferralCode) {
                    item {
                        Spacer8()
                        Box(modifier = Modifier.padding(horizontal = sceneContentPadding())) {
                            MainActionButton(
                                title = stringResource(R.string.rewards_activate_referral_code_title),
                                colors = mainActionButtonColors(
                                    containerColor = Color.White,
                                    contentColor = Color.Black,
                                ),
                            ) { codeDialogShow = true }
                        }
                        Spacer8()
                        Text(
                            modifier = Modifier.fillMaxWidth(),
                            text = stringResource(R.string.rewards_activate_referral_code_description),
                            color = MaterialTheme.colorScheme.secondary,
                            style = MaterialTheme.typography.bodyMedium,
                            textAlign = TextAlign.Center,
                        )
                    }
                }
                uiState.errorNotice?.let { notice ->
                    item { GemListRowView(row = notice, listPosition = ListPosition.Single) }
                }
                uiState.statusNotice?.let { notice ->
                    item {
                        val code = uiState.usedReferralCode?.takeIf { uiState.showsPendingActivation }
                        GemListRowView(row = notice, listPosition = if (code != null) ListPosition.First else ListPosition.Single)
                        if (code != null) {
                            Box(modifier = Modifier.listItem(ListPosition.Last).padding(paddingDefault)) {
                                MainActionButton(
                                    title = stringResource(R.string.transfer_confirm),
                                    state = buttonState(enabled = uiState.canActivatePendingReferral),
                                ) {
                                    onCode(code, onCodeResult)
                                }
                            }
                        }
                    }
                }
                if (uiState.showsInfo) {
                    referralInfo(infoRows, redemptions, onRedeem)
                }
            }
        }
    }

    GetStartedDialog(isVisible = getStartedDialogShow, onUsername = onUsername) {
        getStartedDialogShow = false
    }

    ReferralCodeDialog(
        isVisible = codeDialogShow,
        referralCode = referralCode,
        onCode = onCode,
    ) {
        codeDialogShow = false
        onCancelCode()
    }
}

@Preview
@Composable
private fun ReferralScenePreview() {
    WalletTheme {
        ReferralScene(
            inSync = SyncType.None,
            isAvailableWalletSelect = false,
            referralLink = null,
            uiState = previewRewardsState(
                hasReferralCode = true,
                canInvite = true,
                showsInfo = true,
                referralCode = "testuser",
            ),
            infoRows = emptyList(),
            redemptions = emptyList(),
            currentWallet = previewWallet(),
            onUsername = { _, _ -> },
            onCode = { _, _ -> },
            onCancelCode = {},
            onRefresh = {},
            onWallet = {},
            onRedeem = {},
            onClose = {},
        )
    }
}

@Preview
@Composable
private fun ReferralSceneNoRewardsPreview() {
    WalletTheme {
        ReferralScene(
            inSync = SyncType.None,
            isAvailableWalletSelect = false,
            referralLink = null,
            uiState = previewRewardsState(canUseReferralCode = true),
            infoRows = emptyList(),
            redemptions = emptyList(),
            currentWallet = previewWallet(),
            onUsername = { _, _ -> },
            onCode = { _, _ -> },
            onCancelCode = {},
            onRefresh = {},
            onWallet = {},
            onRedeem = {},
            onClose = {},
        )
    }
}

private fun previewWallet() = Wallet(
    id = WalletId("1"),
    name = "Wallet 1",
    index = 0,
    type = WalletType.Multicoin,
    accounts = emptyList(),
    isPinned = false,
    imageUrl = null,
    source = WalletSource.Create,
)
