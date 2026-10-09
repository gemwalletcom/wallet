package com.gemwallet.android.features.rewards.presents

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
import com.gemwallet.android.features.rewards.presents.components.rewardsHead
import com.gemwallet.android.features.rewards.presents.components.rewardsInfo
import com.gemwallet.android.features.rewards.presents.dialogs.CreateRewardsCodeDialog
import com.gemwallet.android.features.rewards.presents.dialogs.RedeemRewardsCodeDialog
import com.gemwallet.android.features.rewards.viewmodels.models.RewardsSectionUIModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.mainActionButtonColors
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import kotlinx.coroutines.launch
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRewardsIntroItem
import uniffi.gemstone.GemRewardsInviteAction
import uniffi.gemstone.GemRewardsPendingReferral
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsWallet

private val referralCodeMaxWidth = 250.dp

@Composable
fun RewardsScene(
    state: GemLoadState,
    isRefreshing: Boolean,
    wallet: GemRewardsWallet?,
    introItems: List<GemRewardsIntroItem>,
    inviteAction: GemRewardsInviteAction?,
    canUseReferralCode: Boolean,
    pendingReferral: GemRewardsPendingReferral?,
    notices: List<GemListRow>,
    inviteDescription: String,
    sections: List<RewardsSectionUIModel>,
    redemptions: List<GemRewardsRedemption>,
    incomingCode: GemIncomingCode? = null,
    onUsername: (String, (Throwable?) -> Unit) -> Unit,
    onCode: (String, (Throwable?) -> Unit) -> Unit,
    onCodeHandled: () -> Unit,
    onRefresh: () -> Unit,
    onWallet: () -> Unit,
    onRedeem: (GemRewardsRedemption) -> Unit,
    onClose: () -> Unit,
    onInvite: () -> Unit,
    onError: (Throwable) -> Unit,
    snackbar: SnackbarHostState = remember { SnackbarHostState() },
) {
    val context = LocalContext.current

    var getStartedDialogShow by remember(inviteAction) { mutableStateOf(false) }
    var codeDialogShow by remember { mutableStateOf(false) }
    var codeRetryShow by remember { mutableStateOf(false) }
    val referralCode = when (incomingCode) {
        is GemIncomingCode.Activate -> incomingCode.code
        is GemIncomingCode.Confirm -> incomingCode.code
        null -> null
    }

    val successStr = stringResource(R.string.common_done)
    val scope = rememberCoroutineScope()

    val onCodeResult = fun (error: Throwable?) {
        when (error) {
            null -> scope.launch { snackbar.showSnackbar(successStr, R.drawable.ic_check_circle) }
            else -> onError(error)
        }
    }

    LaunchedEffect(incomingCode) {
        val code = (incomingCode as? GemIncomingCode.Activate)?.code ?: return@LaunchedEffect
        context.requestAuth(AuthRequest.Default, onCancel = { codeRetryShow = true }) {
            onCodeHandled()
            onCode(code, onCodeResult)
        }
    }

    Scene(
        title = stringResource(R.string.rewards_title),
        snackbar = snackbar,
        actions = {
            if (wallet?.canChoose == true) {
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
                        text = wallet.row.name,
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
        if (state == GemLoadState.Loading) {
            Box(modifier = Modifier.fillMaxSize()) {
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            }
            return@Scene
        }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                when (state) {
                    GemLoadState.NoData -> {
                        item { EmptyContentView(kind = GemEmptyStateKind.REWARDS, modifier = Modifier.fillParentMaxSize()) }
                        return@LazyColumn
                    }

                    is GemLoadState.Error -> {
                        item { GemListRowView(row = GemListRow.Error(state.error), listPosition = ListPosition.Single) }
                        return@LazyColumn
                    }

                    GemLoadState.Data, GemLoadState.Loading -> Unit
                }
                rewardsHead(
                    description = inviteDescription,
                    intro = introItems,
                    action = inviteAction,
                    onGetStarted = { getStartedDialogShow = true },
                    onShare = onInvite,
                )

                if (canUseReferralCode) {
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
                val pending = pendingReferral
                notices.forEachIndexed { index, notice ->
                    val isLast = index == notices.lastIndex
                    item {
                        GemListRowView(row = notice, listPosition = if (isLast && pending != null) ListPosition.First else ListPosition.Single)
                        if (isLast && pending != null) {
                            Box(modifier = Modifier.listItem(ListPosition.Last).padding(paddingDefault)) {
                                MainActionButton(
                                    title = stringResource(R.string.transfer_confirm),
                                    state = buttonState(enabled = pending.isEnabled),
                                ) {
                                    context.requestAuth(AuthRequest.Default) { onCode(pending.code, onCodeResult) }
                                }
                            }
                        }
                    }
                }
                rewardsInfo(sections, redemptions, onRedeem)
            }
        }
    }

    CreateRewardsCodeDialog(isVisible = getStartedDialogShow, onUsername = onUsername) {
        getStartedDialogShow = false
    }

    RedeemRewardsCodeDialog(
        isVisible = codeDialogShow || codeRetryShow || incomingCode is GemIncomingCode.Confirm,
        referralCode = referralCode,
        onCode = onCode,
    ) {
        codeDialogShow = false
        codeRetryShow = false
        onCodeHandled()
    }
}

@Preview
@Composable
private fun RewardsScenePreview() {
    WalletTheme {
        RewardsScene(
            state = GemLoadState.Data,
            isRefreshing = false,
            wallet = null,
            introItems = GemRewardsIntroItem.entries,
            inviteAction = GemRewardsInviteAction.SHARE,
            canUseReferralCode = false,
            pendingReferral = null,
            notices = emptyList(),
            inviteDescription = "Invite friends and earn 100 points",
            sections = emptyList(),
            redemptions = emptyList(),
            onUsername = { _, _ -> },
            onCode = { _, _ -> },
            onCodeHandled = {},
            onRefresh = {},
            onWallet = {},
            onRedeem = {},
            onClose = {},
            onInvite = {},
            onError = {},
        )
    }
}

@Preview
@Composable
private fun RewardsSceneNoRewardsPreview() {
    WalletTheme {
        RewardsScene(
            state = GemLoadState.Data,
            isRefreshing = false,
            wallet = null,
            introItems = GemRewardsIntroItem.entries,
            inviteAction = GemRewardsInviteAction.CREATE_CODE,
            canUseReferralCode = true,
            pendingReferral = null,
            notices = emptyList(),
            inviteDescription = "Invite friends and earn 100 points",
            sections = emptyList(),
            redemptions = emptyList(),
            onUsername = { _, _ -> },
            onCode = { _, _ -> },
            onCodeHandled = {},
            onRefresh = {},
            onWallet = {},
            onRedeem = {},
            onClose = {},
            onInvite = {},
            onError = {},
        )
    }
}
