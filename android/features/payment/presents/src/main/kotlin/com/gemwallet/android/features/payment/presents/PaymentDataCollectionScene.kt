package com.gemwallet.android.features.payment.presents

import android.annotation.SuppressLint
import android.webkit.JavascriptInterface
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.viewinterop.AndroidView
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.Scene
import org.json.JSONObject

private const val MESSAGE_HANDLER = "payDataCollectionComplete"
private const val MESSAGE_TYPE_KEY = "type"
private const val MESSAGE_ERROR_KEY = "error"
private const val COMPLETE = "IC_COMPLETE"
private const val ERROR = "IC_ERROR"
private const val ALLOWED_HOST = "walletconnect.com"

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun PaymentDataCollectionScene(
    url: String,
    onComplete: () -> Unit,
    onError: (String?) -> Unit,
    onCancel: () -> Unit,
) {
    var webView by remember { mutableStateOf<WebView?>(null) }

    BackHandler {
        val view = webView
        if (view != null && view.canGoBack()) view.goBack() else onCancel()
    }

    Scene(
        title = stringResource(R.string.transfer_payment_title),
        onClose = onCancel,
    ) {
        AndroidView(
            modifier = Modifier.fillMaxSize(),
            factory = { context ->
                WebView(context).apply {
                    settings.javaScriptEnabled = true
                    settings.domStorageEnabled = true
                    webViewClient = AllowedHostWebViewClient()
                    addJavascriptInterface(CollectDataBridge(onComplete, onError), MESSAGE_HANDLER)
                    loadUrl(url)
                    webView = this
                }
            },
        )
    }
}

private class CollectDataBridge(
    private val onComplete: () -> Unit,
    private val onError: (String?) -> Unit,
) {
    @JavascriptInterface
    fun postMessage(payload: String) {
        val message = runCatching { JSONObject(payload) }.getOrNull() ?: return
        when (message.optString(MESSAGE_TYPE_KEY)) {
            COMPLETE -> onComplete()
            ERROR -> onError(message.optString(MESSAGE_ERROR_KEY).takeIf { it.isNotEmpty() })
        }
    }
}

private class AllowedHostWebViewClient : WebViewClient() {
    override fun shouldOverrideUrlLoading(view: WebView?, request: android.webkit.WebResourceRequest?): Boolean {
        val uri = request?.url ?: return true
        val host = uri.host?.lowercase() ?: return true
        val allowed = uri.scheme == "https" && (host == ALLOWED_HOST || host.endsWith(".$ALLOWED_HOST"))
        return !allowed
    }

    override fun onPageStarted(view: WebView?, url: String?, favicon: android.graphics.Bitmap?) {
        view?.evaluateJavascript(BRIDGE_SHIM, null)
    }

    override fun onPageFinished(view: WebView?, url: String?) {
        view?.evaluateJavascript(BRIDGE_SHIM, null)
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
