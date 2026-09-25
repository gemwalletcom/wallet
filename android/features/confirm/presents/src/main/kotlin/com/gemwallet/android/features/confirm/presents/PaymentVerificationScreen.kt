package com.gemwallet.android.features.confirm.presents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.confirm.viewmodels.PaymentVerificationViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.WebView
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import uniffi.gemstone.GemInfoTopic

@Composable
fun PaymentVerificationScreen(onCancel: () -> Unit, onConfirm: ConfirmTransactionAction, viewModel: PaymentVerificationViewModel = hiltViewModel()) {
    val url by viewModel.url.collectAsStateWithLifecycle()
    val confirm by viewModel.confirm.collectAsStateWithLifecycle()
    val isFailed by viewModel.isFailed.collectAsStateWithLifecycle()
    val snackbar = remember { SnackbarHostState() }
    var isInfoVisible by remember { mutableStateOf(false) }
    ToastEffect(viewModel.toastEvents, snackbar)

    LaunchedEffect(confirm) {
        confirm?.let { onConfirm(it) }
    }

    Scene(
        title = stringResource(R.string.info_payment_verification_title),
        closeIcon = true,
        onClose = onCancel,
        snackbar = snackbar,
        actions = {
            IconButton(onClick = { isInfoVisible = true }) {
                Icon(AppIcons.InfoOutlined, "")
            }
        },
    ) {
        WebView(
            url = url,
            bridge = viewModel.verificationBridge,
            modifier = Modifier.fillMaxSize(),
        )
    }

    if (isInfoVisible) {
        InfoBottomSheet(GemInfoTopic.PaymentVerification.infoSheet()) { isInfoVisible = false }
    }

    if (isFailed) {
        AlertDialog(
            onDismissRequest = onCancel,
            confirmButton = {
                Button(onCancel) { Text(stringResource(R.string.common_done)) }
            },
            text = {
                Text(stringResource(R.string.errors_error_occurred))
            },
        )
    }
}
