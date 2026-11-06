package com.gemwallet.android.features.transfer.presents.receive

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.LifecycleStartEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.transfer.presents.receive.components.ReceiveQrCard
import com.gemwallet.android.features.transfer.presents.receive.components.receiveQrPadding
import com.gemwallet.android.features.transfer.presents.receive.components.receiveQrWidth
import com.gemwallet.android.features.transfer.viewmodels.receive.LightningReceiveViewModel
import com.gemwallet.android.features.transfer.viewmodels.receive.LightningReceiveViewModel.State
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.secondaryActionButtonColors
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.fields.AmountField
import com.gemwallet.android.ui.components.fields.AmountSymbolPlacement
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
fun LightningReceiveScreen(onCancel: () -> Unit) {
    val viewModel: LightningReceiveViewModel = hiltViewModel()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val amount by viewModel.amount.collectAsStateWithLifecycle()
    val equivalent by viewModel.equivalent.collectAsStateWithLifecycle()
    val context = LocalContext.current
    val title = stringResource(R.string.wallet_receive)
    val snackbar = remember { SnackbarHostState() }
    ToastEffect(viewModel.toastEvents, snackbar)

    LifecycleStartEffect(viewModel) {
        viewModel.onVisible()
        onStopOrDispose { viewModel.onHidden() }
    }
    LaunchedEffect(state) {
        when (state) {
            State.Locked -> context.requestAuth(AuthRequest.Default, onCancel = viewModel::onAuthCancelled, onSuccess = viewModel::onAuthenticated)
            State.Received -> onCancel()
            else -> Unit
        }
    }

    when (val current = state) {
        State.Opening, State.Locked, State.Received -> LoadingScene(title = title, onCancel = onCancel)

        is State.Failed -> Scene(
            title = title,
            onClose = onCancel,
            mainAction = { MainActionButton(title = stringResource(R.string.common_try_again), onClick = viewModel::onRetry) },
        ) {
            LightningMessage(current.message)
        }

        is State.Amount -> LightningAmountScene(
            title = title,
            amount = amount,
            amountSymbol = viewModel.amountSymbol,
            equivalent = equivalent,
            isCreating = current.isCreating,
            snackbar = snackbar,
            onAmount = viewModel::onAmount,
            onCreate = viewModel::onCreateInvoice,
            onCancel = onCancel,
        )

        is State.Invoice -> LightningInvoiceScene(title = title, invoice = current, onChangeAmount = viewModel::onChangeAmount, onCancel = onCancel)

        is State.Expired -> Scene(
            title = title,
            onClose = onCancel,
            mainAction = { MainActionButton(title = stringResource(R.string.receive_lightning_change_amount), onClick = viewModel::onChangeAmount) },
        ) {
            LightningMessage(current.amount, stringResource(R.string.transaction_status_expired))
        }
    }
}

@Composable
private fun LightningAmountScene(title: String, amount: String, amountSymbol: String, equivalent: String, isCreating: Boolean, snackbar: SnackbarHostState, onAmount: (String) -> Unit, onCreate: () -> Unit, onCancel: () -> Unit) {
    Scene(
        title = title,
        onClose = onCancel,
        snackbar = snackbar,
        mainAction = {
            MainActionButton(
                title = stringResource(R.string.receive_lightning_create_invoice),
                state = if (isCreating) ButtonState.Loading else ButtonState.Enabled,
                onClick = onCreate,
            )
        },
    ) {
        Spacer(modifier = Modifier.size(paddingDefault))
        Column(modifier = Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            AmountField(
                amount = amount,
                symbol = AmountSymbolUIModel(amountSymbol, AmountSymbolPlacement.Trailing),
                equivalent = equivalent,
                onValueChange = onAmount,
                onNext = onCreate,
                keyboardType = KeyboardType.Number,
                maximumFractionDigits = 0,
                error = "",
            )
            Spacer(modifier = Modifier.size(paddingDefault))
            Text(
                modifier = Modifier.padding(horizontal = paddingDefault),
                text = stringResource(R.string.receive_lightning_amount_hint),
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
private fun LightningInvoiceScene(title: String, invoice: State.Invoice, onChangeAmount: () -> Unit, onCancel: () -> Unit) {
    val context = LocalContext.current
    val clipboardManager = context.clipboardManager()
    val shareTitle = stringResource(R.string.common_share)
    val onCopy = { clipboardManager.setPlainText(context, invoice.invoice) }

    Scene(
        title = title,
        onClose = onCancel,
        actions = {
            IconButton({ context.shareText(subject = null, text = invoice.invoice, chooserTitle = shareTitle) }) {
                Icon(AppIcons.Share, shareTitle)
            }
        },
        mainAction = {
            Column(verticalArrangement = Arrangement.spacedBy(paddingDefault)) {
                MainActionButton(title = stringResource(R.string.common_copy), icon = AppIcons.ContentCopy, onClick = onCopy)
                MainActionButton(title = stringResource(R.string.receive_lightning_change_amount), colors = secondaryActionButtonColors(), onClick = onChangeAmount)
            }
        },
    ) {
        Spacer(modifier = Modifier.weight(1f))
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(receiveQrPadding()),
        ) {
            Text(text = invoice.amount, style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.SemiBold)
            ReceiveQrCard(content = invoice.payload, caption = invoice.shortInvoice, onCopy = onCopy)
            Text(
                modifier = Modifier.width(receiveQrWidth()),
                text = invoice.hint,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        Spacer(modifier = Modifier.weight(1f))
    }
}

@Composable
private fun ColumnScope.LightningMessage(title: String, subtitle: String? = null) {
    Spacer(modifier = Modifier.weight(1f))
    Column(
        modifier = Modifier.fillMaxWidth().padding(paddingDefault),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(paddingDefault),
    ) {
        Text(text = title, textAlign = TextAlign.Center, color = MaterialTheme.colorScheme.secondary)
        subtitle?.let { Text(text = it, textAlign = TextAlign.Center, color = MaterialTheme.colorScheme.secondary, style = MaterialTheme.typography.bodyMedium) }
    }
    Spacer(modifier = Modifier.weight(1f))
}
