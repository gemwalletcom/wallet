package com.gemwallet.android.features.transfer.presents.receive

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.transfer.presents.receive.components.ReceiveQrCard
import com.gemwallet.android.features.transfer.presents.receive.components.receiveQrPadding
import com.gemwallet.android.features.transfer.viewmodels.receive.LightningReceiveViewModel
import com.gemwallet.android.features.transfer.viewmodels.receive.LightningReceiveViewModel.State
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.ObserveStartedState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.fields.AmountField
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.ui.style.amountSymbol
import com.gemwallet.android.ui.style.keyboardType
import uniffi.gemstone.GemLightningReceive

@Composable
fun LightningReceiveScreen(onCancel: () -> Unit) {
    val viewModel: LightningReceiveViewModel = hiltViewModel()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val amount by viewModel.amount.collectAsStateWithLifecycle()
    val context = LocalContext.current
    val title = stringResource(R.string.wallet_receive)
    val unlock = { context.requestAuth(AuthRequest.Default, onCancel = {}, onSuccess = viewModel::onRetry) }

    ObserveStartedState(viewModel::setRefreshEnabled)
    LaunchedEffect(state) {
        when (state) {
            State.Locked -> unlock()
            State.Received -> onCancel()
            State.Loading, State.Unavailable, is State.Ready -> Unit
        }
    }

    when (val current = state) {
        State.Loading, State.Received -> LoadingScene(title = title, onCancel = onCancel)
        State.Locked -> Scene(
            title = title,
            onClose = onCancel,
            mainAction = { MainActionButton(title = stringResource(R.string.lock_unlock), onClick = unlock) },
        ) {}
        State.Unavailable -> FatalStateScene(title = title, message = stringResource(R.string.common_not_available), onCancel = onCancel, onTryAgain = viewModel::onRetry)
        is State.Ready -> LightningReceiveScene(title = title, receive = current.receive, amount = amount, onAmount = viewModel::onAmount, onCancel = onCancel)
    }
}

@Composable
private fun LightningReceiveScene(title: String, receive: GemLightningReceive, amount: String, onAmount: (String) -> Unit, onCancel: () -> Unit) {
    val context = LocalContext.current
    val clipboardManager = context.clipboardManager()
    val shareTitle = stringResource(R.string.common_share)
    val onCopy = { clipboardManager.setCopy(context, receive.copy) }

    Scene(
        title = title,
        onClose = onCancel,
        actions = {
            IconButton({ context.shareText(subject = null, text = receive.copy.value, chooserTitle = shareTitle) }) {
                Icon(AppIcons.Share, shareTitle)
            }
        },
        mainAction = { MainActionButton(title = stringResource(R.string.common_copy), icon = AppIcons.ContentCopy, onClick = onCopy) },
    ) {
        Spacer(modifier = Modifier.weight(1f))
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(receiveQrPadding()),
        ) {
            AmountField(
                amount = amount,
                symbol = receive.field.amountSymbol(),
                equivalent = "",
                onValueChange = onAmount,
                onNext = {},
                keyboardType = receive.field.keyboard.keyboardType(),
                error = "",
            )
            ReceiveQrCard(content = receive.qr, caption = receive.copy.display, onCopy = onCopy)
        }
        Spacer(modifier = Modifier.weight(1f))
    }
}
