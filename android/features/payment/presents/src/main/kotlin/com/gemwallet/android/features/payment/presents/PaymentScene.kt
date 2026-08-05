package com.gemwallet.android.features.payment.presents

import android.widget.Toast
import android.widget.Toast.makeText
import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.features.payment.viewmodels.PaymentError
import com.gemwallet.android.features.payment.viewmodels.PaymentSceneState
import com.gemwallet.android.features.payment.viewmodels.model.PaymentQuoteUIModel
import com.gemwallet.android.features.payment.viewmodels.PaymentViewModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentOutcomeUIModel
import com.gemwallet.android.model.AuthRequest
import com.wallet.core.primitives.AssetId
import com.gemwallet.android.ui.R
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.CenteredListHeadSubtitleLayout
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyExpiryItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.walletItemIconModel
import com.gemwallet.android.ui.components.message.SignMessageFullMessageSheet
import com.gemwallet.android.ui.components.message.SignMessageSheetType
import com.gemwallet.android.ui.components.message.SignMessagePayloadDetailsSheet
import com.gemwallet.android.ui.components.message.signMessageText
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.PaymentException

@Composable
fun PaymentScene(
    provider: GemPaymentProviderName,
    paymentId: String,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    onCancel: () -> Unit,
    viewModel: PaymentViewModel = hiltViewModel(),
) {
    val state by viewModel.sceneState.collectAsStateWithLifecycle()

    LaunchedEffect(paymentId) { viewModel.onPayment(provider, paymentId) }

    when (val sceneState = state) {
        PaymentSceneState.Loading -> LoadingScene(
            title = stringResource(R.string.transfer_payment_title),
            onCancel = onCancel,
        )
        is PaymentSceneState.Quotes -> PaymentQuotesScene(
            state = sceneState,
            onSelect = viewModel::onSelectQuote,
            onConfirm = viewModel::onConfirmQuote,
            onCancel = onCancel,
        )
        is PaymentSceneState.CollectData -> PaymentDataCollectionScene(
            url = sceneState.url,
            onComplete = viewModel::onDataCollected,
            onError = viewModel::onDataCollectionError,
            onCancel = onCancel,
        )
        is PaymentSceneState.Approve -> ConfirmScreen(
            params = sceneState.params,
            finishAction = { hash -> viewModel.onActionResult(hash) },
            cancelAction = onCancel,
            onAcquireAsset = onAcquireAsset,
        )
        is PaymentSceneState.Confirm -> ConfirmScreen(
            params = sceneState.params,
            finishAction = { hash -> viewModel.onActionResult(hash) },
            cancelAction = onCancel,
            onAcquireAsset = onAcquireAsset,
        )
        is PaymentSceneState.SignMessage -> PaymentSignMessageScene(
            state = sceneState,
            onApprove = viewModel::onSign,
            onCancel = onCancel,
        )
        is PaymentSceneState.Outcome -> PaymentToastEffect(sceneState.outcome.messageRes(), onCancel)
        is PaymentSceneState.Error -> PaymentToastEffect(sceneState.error.messageRes(), onCancel)
    }
}

@Composable
private fun PaymentQuotesScene(
    state: PaymentSceneState.Quotes,
    onSelect: (String) -> Unit,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    var isSelectingQuote by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(R.string.transfer_payment_title),
        backHandle = true,
        onClose = onCancel,
        mainAction = {
            MainActionButton(
                title = stringResource(R.string.common_continue),
                state = if (state.expired || state.selected == null) ButtonState.Disabled else ButtonState.Enabled,
                onClick = onConfirm,
            )
        },
    ) {
        LazyColumn {
            item {
                CenteredListHead(
                    icon = state.merchant.iconUrl,
                    title = state.price ?: state.selectedQuote?.amountText.orEmpty(),
                    placeholderText = state.merchant.name.firstOrNull()?.uppercaseChar()?.toString(),
                )
            }
            item {
                PropertyItem(
                    title = { PropertyTitleText(R.string.transfer_merchant) },
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
                    listPosition = if (state.expiresAt == null) ListPosition.Last else ListPosition.Middle,
                )
            }
            state.expiresAt?.let { expiresAt ->
                item {
                    PropertyExpiryItem(
                        title = stringResource(R.string.transfer_payment_expires_in),
                        expiresAt = expiresAt,
                        listPosition = ListPosition.Last,
                    )
                }
            }
            item {
                PropertyItem(
                    modifier = Modifier.clickable { if (!state.expired) isSelectingQuote = true },
                    title = { PropertyTitleText(R.string.transfer_pay_with) },
                    data = {
                        PropertyDataText(
                            text = state.selectedQuote?.amountText.orEmpty(),
                            badge = { DataBadgeChevron(icon = state.selectedQuote?.iconUrl.orEmpty()) },
                        )
                    },
                    listPosition = ListPosition.Single,
                )
            }
        }
    }

    PaymentQuotesSelectModal(
        isVisible = isSelectingQuote,
        quotes = state.quotes,
        selected = state.selected,
        onSelect = {
            onSelect(it)
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
private fun PaymentSignMessageScene(
    state: PaymentSceneState.SignMessage,
    onApprove: () -> Unit,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    var sheetType by remember { mutableStateOf<SignMessageSheetType?>(null) }

    Scene(
        title = stringResource(R.string.transfer_payment_title),
        backHandle = true,
        closeIcon = true,
        onClose = onCancel,
        mainAction = {
            MainActionButton(title = stringResource(R.string.transfer_confirm)) {
                context.requestAuth(AuthRequest.Confirmation) { onApprove() }
            }
        },
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding() + paddingDefault),
        ) {
            item {
                CenteredListHead(
                    icon = state.quote?.iconUrl ?: state.merchant.iconUrl,
                    title = state.quote?.amountText ?: state.merchant.name,
                    subtitle = state.price,
                    placeholderText = state.merchant.name.firstOrNull()?.uppercaseChar()?.toString(),
                    subtitleLayout = CenteredListHeadSubtitleLayout.Vertical,
                )
            }
            item { PropertyItem(R.string.transfer_merchant, state.merchant.name, listPosition = ListPosition.First) }
            item { PropertyItem(R.string.common_wallet, state.walletName, listPosition = ListPosition.Middle) }
            item {
                PropertyNetworkItem(
                    state.chain,
                    listPosition = if (state.expiresAt == null) ListPosition.Last else ListPosition.Middle,
                )
            }
            state.expiresAt?.let { expiresAt ->
                item {
                    PropertyExpiryItem(
                        title = stringResource(R.string.transfer_payment_expires_in),
                        expiresAt = expiresAt,
                        listPosition = ListPosition.Last,
                    )
                }
            }
            if (state.quote == null) {
                if (state.hasPayload) {
                    simulationPayloadFieldsContent(
                        fields = state.primaryPayloadFields,
                        onDetailsClick = { sheetType = SignMessageSheetType.Details },
                    )
                } else {
                    signMessageText(state.plainMessage)
                }
            }
        }

        when (sheetType) {
            SignMessageSheetType.Details -> SignMessagePayloadDetailsSheet(
                primaryFields = state.primaryPayloadFields,
                secondaryFields = state.secondaryPayloadFields,
                onViewFullMessage = { sheetType = SignMessageSheetType.FullMessage },
                onDismissRequest = { sheetType = null },
            )
            SignMessageSheetType.FullMessage -> SignMessageFullMessageSheet(
                message = state.plainMessage,
                onDismissRequest = { sheetType = null },
            )
            null -> Unit
        }
    }
}

@Composable
private fun PaymentToastEffect(
    @StringRes message: Int?,
    onDismiss: () -> Unit,
) {
    val context = LocalContext.current
    LaunchedEffect(message) {
        message?.let { makeText(context, it, Toast.LENGTH_SHORT).show() }
        onDismiss()
    }
}

private fun PaymentError.messageRes(): Int = when (this) {
    PaymentError.NoWallet,
    PaymentError.NoQuotes,
    PaymentError.QuoteUnavailable,
    PaymentError.NoAccount -> R.string.errors_not_supported
    PaymentError.WatchWallet -> R.string.wallet_watch_tooltip_title
    PaymentError.DataCollection -> R.string.errors_error_occurred
    PaymentError.UnknownAsset -> R.string.errors_error_occurred
    is PaymentError.Gateway -> error.messageRes()
}

private fun PaymentException?.messageRes(): Int = when (this) {
    is PaymentException.PaymentExpired,
    is PaymentException.QuoteExpired -> R.string.errors_payment_expired
    is PaymentException.Rejected -> R.string.errors_payment_not_allowed
    is PaymentException.PaymentNotFound,
    is PaymentException.RateLimited -> R.string.transaction_status_failed
    is PaymentException.NoPaymentOptions,
    is PaymentException.UnsupportedAccounts,
    is PaymentException.NotSupported -> R.string.errors_not_supported
    is PaymentException.InvalidRequest,
    is PaymentException.Network,
    null -> R.string.errors_error_occurred
}

private fun PaymentOutcomeUIModel.messageRes(): Int? = when (this) {
    PaymentOutcomeUIModel.Success -> R.string.transaction_status_confirmed
    PaymentOutcomeUIModel.Pending -> R.string.transaction_status_pending
    PaymentOutcomeUIModel.Cancelled -> null
    PaymentOutcomeUIModel.Expired -> R.string.errors_payment_expired
    PaymentOutcomeUIModel.Failed -> R.string.transaction_status_failed
}
