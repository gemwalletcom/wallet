package com.gemwallet.android.features.payment.presents.components

import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.payment.presents.PaymentSceneAction
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.web.WebView
import com.gemwallet.android.ui.components.web.WebViewMessageHandler

private const val MESSAGE_HANDLER = "payDataCollectionComplete"
private const val MESSAGE_TYPE_KEY = "type"
private const val COMPLETE = "IC_COMPLETE"

@Composable
internal fun PaymentDataCollectionModal(
    url: String?,
    onAction: (PaymentSceneAction) -> Unit,
) {
    ModalBottomSheet(
        isVisible = url != null,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.transfer_payment_title),
        onDismissRequest = { onAction(PaymentSceneAction.DismissDataCollection) },
    ) {
        url ?: return@ModalBottomSheet
        WebView(
            url = url,
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(),
            messageHandler = WebViewMessageHandler(
                name = MESSAGE_HANDLER,
                script = BRIDGE_SHIM,
            ) { message ->
                if (message.optString(MESSAGE_TYPE_KEY) == COMPLETE) onAction(PaymentSceneAction.DataCollected)
            },
            onBack = { onAction(PaymentSceneAction.DismissDataCollection) },
        )
    }
}

private val BRIDGE_SHIM = """
(function() {
    if (window.__gemPayBridge) { return; }
    window.__gemPayBridge = true;
    var android = $MESSAGE_HANDLER;
    var post = function(message) {
        if (message === null || message === undefined) { return; }
        try {
            android.postMessage(typeof message === 'string' ? message : JSON.stringify(message));
        } catch (error) {}
    };
    window.webkit = window.webkit || {};
    window.webkit.messageHandlers = window.webkit.messageHandlers || {};
    window.webkit.messageHandlers.$MESSAGE_HANDLER = { postMessage: post };
    window.addEventListener('message', function(event) { post(event.data); });
})();
""".trimIndent()
