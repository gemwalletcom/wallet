package com.gemwallet.android.features.payment.presents

import android.widget.Toast
import android.widget.Toast.makeText
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.features.payment.viewmodels.PaymentLinkError
import com.gemwallet.android.features.payment.viewmodels.PaymentSceneState
import com.gemwallet.android.features.payment.viewmodels.PaymentViewModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentOutcomeUIModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentQuoteUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.walletItemIconModel
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PaymentLink
import uniffi.gemstone.PaymentException

@Composable
fun PaymentScreen(
    link: PaymentLink,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    onCancel: () -> Unit,
    viewModel: PaymentViewModel = hiltViewModel(),
) {
    val state by viewModel.sceneState.collectAsStateWithLifecycle()

    LaunchedEffect(link) { viewModel.onPayment(link) }

    val onAction: (PaymentSceneAction) -> Unit = { action ->
        when (action) {
            is PaymentSceneAction.SelectQuote -> viewModel.onSelectQuote(action.quoteId)
            PaymentSceneAction.ConfirmQuote -> viewModel.onConfirmQuote()
            PaymentSceneAction.DataCollected -> viewModel.onDataCollected()
            is PaymentSceneAction.DataCollectionFailed -> viewModel.onDataCollectionError(action.message)
            PaymentSceneAction.DismissDataCollection -> viewModel.onDismissDataCollection()
            PaymentSceneAction.BackFromConfirm -> viewModel.onBackFromConfirm()
            is PaymentSceneAction.TransactionHash -> viewModel.onTransactionHash(action.hash)
            PaymentSceneAction.Retry -> viewModel.onRetry()
            PaymentSceneAction.Cancel -> onCancel()
        }
    }

    AnimatedContent(
        targetState = state,
        contentKey = { it is PaymentSceneState.Confirm },
        transitionSpec = { navigationSlideTransition(forward = targetState is PaymentSceneState.Confirm) },
        label = "payment",
    ) { sceneState ->
    when (sceneState) {
        PaymentSceneState.Loading -> LoadingScene(
            title = stringResource(R.string.transfer_payment_title),
            onCancel = { onAction(PaymentSceneAction.Cancel) },
        )
        is PaymentSceneState.Quotes -> PaymentQuotesScene(
            state = sceneState,
            onAction = onAction,
        )
        is PaymentSceneState.Confirm -> ConfirmScreen(
            params = sceneState.params,
            finishAction = { hash -> onAction(PaymentSceneAction.TransactionHash(hash)) },
            cancelAction = { onAction(PaymentSceneAction.BackFromConfirm) },
            onAcquireAsset = onAcquireAsset,
            handleSystemBack = true,
        )
        is PaymentSceneState.Outcome -> PaymentToastEffect(sceneState.outcome.message()) { onAction(PaymentSceneAction.Cancel) }
        PaymentSceneState.Done -> PaymentToastEffect(null) { onAction(PaymentSceneAction.Cancel) }
        is PaymentSceneState.Error -> PaymentToastEffect(sceneState.error.message()) { onAction(PaymentSceneAction.Cancel) }
    }
    }
}

@Composable
private fun PaymentQuotesScene(
    state: PaymentSceneState.Quotes,
    onAction: (PaymentSceneAction) -> Unit,
) {
    var isSelectingQuote by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(R.string.transfer_payment_title),
        backHandle = true,
        onClose = { onAction(PaymentSceneAction.Cancel) },
        mainAction = {
            MainActionButton(
                    state = if (state.selected == null) ButtonState.Disabled else ButtonState.Enabled,
                    onClick = { onAction(PaymentSceneAction.ConfirmQuote) },
                ) {
                    if (state.selectedQuote?.requiresVerification == true) {
                        Icon(AppIcons.Person, contentDescription = null)
                        Spacer8()
                    }
                    Text(
                        text = stringResource(R.string.common_continue),
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Medium,
                    )
                }
        },
    ) {
        LazyColumn {
            item {
                CenteredListHead(
                    icon = state.selectedQuote?.iconUrl ?: state.merchant.iconUrl,
                    title = state.selectedQuote?.amountText ?: state.merchant.name,
                    subtitle = state.price,
                    placeholderText = state.merchant.name.firstOrNull()?.uppercaseChar()?.toString(),
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item {
                PropertyItem(
                    title = { PropertyTitleText(R.string.transfer_recipient_title) },
                    data = {
                        PropertyDataText(
                            text = state.merchant.name,
                            badge = state.merchant.iconUrl?.let {
                                { DataBadgeChevron(icon = it, isShowChevron = false) }
                            },
                        )
                    },
                    listPosition = ListPosition.First,
                )
            }
            item {
                PropertyItem(
                    title = { PropertyTitleText(R.string.common_wallet) },
                    data = {
                        val walletIcon = walletItemIconModel(state.walletType, state.walletChain)
                        PropertyDataText(
                            text = state.walletName,
                            badge = walletIcon?.let { { DataBadgeChevron(icon = it, isShowChevron = false) } },
                        )
                    },
                    listPosition = ListPosition.Last,
                )
            }
            item {
                val isSelectable = state.quotes.size > 1
                PropertyItem(
                    modifier = Modifier.clickable(enabled = isSelectable) { isSelectingQuote = true },
                    title = { PropertyTitleText(R.string.transfer_pay_with) },
                    data = {
                        PropertyDataText(
                            text = state.selectedQuote?.amountText.orEmpty(),
                            badge = { DataBadgeChevron(icon = state.selectedQuote?.iconUrl.orEmpty(), isShowChevron = isSelectable) },
                        )
                    },
                    listPosition = ListPosition.Single,
                )
            }
        }
    }

    PaymentDataCollectionModal(
        url = state.collectData,
        onAction = onAction,
    )

    PaymentQuotesSelectModal(
        isVisible = isSelectingQuote,
        quotes = state.quotes,
        selected = state.selected,
        onSelect = {
            onAction(PaymentSceneAction.SelectQuote(it))
            isSelectingQuote = false
        },
        onDismissRequest = { isSelectingQuote = false },
    )
}

@Composable
private fun PaymentQuotesSelectModal(
    isVisible: Boolean,
    quotes: List<PaymentQuoteUIModel>,
    selected: String?,
    onSelect: (String) -> Unit,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        title = stringResource(R.string.transfer_pay_with),
        onDismissRequest = onDismissRequest,
    ) {
        LazyColumn {
            itemsIndexed(quotes) { index, quote ->
                ListItem(
                    modifier = Modifier.clickable { onSelect(quote.id) },
                    leading = {
                        IconWithBadge(
                            icon = quote.iconUrl,
                            placeholder = quote.symbol,
                            supportIcon = quote.supportIconUrl,
                        )
                    },
                    title = { ListItemTitleText(quote.name) },
                    subtitle = { ListItemSupportText(quote.networkName) },
                    trailing = {
                        getBalanceInfo(quote.amountText, quote.balance, false).invoke()
                        if (quote.id == selected) {
                            Spacer(modifier = Modifier.width(paddingSmall))
                            SelectionCheckmark()
                        }
                    },
                    listPosition = ListPosition.getPosition(index, quotes.size),
                )
            }
        }
    }
}

@Composable
private fun PaymentToastEffect(
    message: String?,
    onDismiss: () -> Unit,
) {
    val context = LocalContext.current
    LaunchedEffect(message) {
        message?.let { makeText(context, it, Toast.LENGTH_SHORT).show() }
        onDismiss()
    }
}

@Composable
private fun PaymentLinkError.message(): String = when (this) {
    PaymentLinkError.NoWallet,
    PaymentLinkError.QuoteUnavailable,
    PaymentLinkError.NoAccount -> stringResource(R.string.errors_not_supported)
    PaymentLinkError.WatchWallet -> stringResource(R.string.wallet_watch_tooltip_title)
    is PaymentLinkError.Gateway -> error.message()
}

@Composable
private fun PaymentException?.message(): String = when (this) {
    is PaymentException.PaymentExpired,
    is PaymentException.QuoteExpired -> stringResource(R.string.errors_payment_expired)
    is PaymentException.Rejected -> stringResource(R.string.errors_payment_not_allowed)
    is PaymentException.PaymentNotFound -> stringResource(R.string.transaction_status_failed)
    is PaymentException.NoPaymentOptions,
    is PaymentException.NotSupported -> stringResource(R.string.errors_not_supported)
    is PaymentException.InvalidRequest,
    is PaymentException.Network,
    null -> stringResource(R.string.errors_error_occurred)
}

@Composable
private fun PaymentOutcomeUIModel.message(): String? = when (this) {
    PaymentOutcomeUIModel.Success -> stringResource(R.string.transaction_status_confirmed)
    PaymentOutcomeUIModel.Pending -> stringResource(R.string.transaction_status_pending)
    PaymentOutcomeUIModel.Cancelled -> null
    PaymentOutcomeUIModel.Expired -> stringResource(R.string.errors_payment_expired)
    PaymentOutcomeUIModel.Failed -> stringResource(R.string.transaction_status_failed)
}
