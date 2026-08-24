package com.gemwallet.android.features.payment.presents

import android.widget.Toast
import android.widget.Toast.makeText
import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.features.payment.presents.components.message
import com.gemwallet.android.features.payment.viewmodels.PaymentSceneState
import com.gemwallet.android.features.payment.viewmodels.PaymentViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PaymentLink

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
            is PaymentSceneState.Outcome -> PaymentToastEffect(sceneState.status.message()) { onAction(PaymentSceneAction.Cancel) }
            PaymentSceneState.Done -> PaymentToastEffect(null) { onAction(PaymentSceneAction.Cancel) }
            is PaymentSceneState.Error -> PaymentToastEffect(sceneState.error.message()) { onAction(PaymentSceneAction.Cancel) }
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
