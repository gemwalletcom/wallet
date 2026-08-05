package com.gemwallet.android.features.payment.presents

import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.ApplicationInfo
import android.graphics.Bitmap
import android.util.Log
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams.MATCH_PARENT
import android.webkit.ConsoleMessage
import android.webkit.CookieManager
import android.webkit.JavascriptInterface
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.viewinterop.AndroidView
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.open
import org.json.JSONObject

private const val MESSAGE_HANDLER = "payDataCollectionComplete"
private const val MESSAGE_TYPE_KEY = "type"
private const val MESSAGE_ERROR_KEY = "error"
private const val COMPLETE = "IC_COMPLETE"
private const val ERROR = "IC_ERROR"
private const val ALLOWED_HOST = "walletconnect.com"
private const val TAG = "PaymentDataCollection"

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun PaymentDataCollectionScene(
    url: String,
    onComplete: () -> Unit,
    onError: (String?) -> Unit,
    onCancel: () -> Unit,
) {
    var webView by remember { mutableStateOf<WebView?>(null) }
    val uriHandler = LocalUriHandler.current

    BackHandler {
        val view = webView
        if (view != null && view.canGoBack()) view.goBack() else onCancel()
    }

    Scene(
        title = stringResource(R.string.transfer_payment_title),
        onClose = onCancel,
    ) {
        AndroidView(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f),
            factory = { context ->
                if (context.applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0) {
                    WebView.setWebContentsDebuggingEnabled(true)
                }
                WebView(context).apply {
                    layoutParams = ViewGroup.LayoutParams(MATCH_PARENT, MATCH_PARENT)
                    settings.javaScriptEnabled = true
                    settings.domStorageEnabled = true
                    settings.useWideViewPort = true
                    settings.loadWithOverviewMode = true
                    CookieManager.getInstance().setAcceptThirdPartyCookies(this, true)
                    webViewClient = AllowedHostWebViewClient(context, uriHandler)
                    webChromeClient = LoggingWebChromeClient()
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

private class AllowedHostWebViewClient(
    private val context: Context,
    private val uriHandler: UriHandler,
) : WebViewClient() {
    override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
        val uri = request?.url ?: return true
        if (uri.scheme != "https" && uri.scheme != "http") return true
        val host = uri.host?.lowercase() ?: return true
        if (uri.scheme == "https" && (host == ALLOWED_HOST || host.endsWith(".$ALLOWED_HOST"))) return false
        uriHandler.open(context, uri.toString())
        return true
    }

    override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
        view?.evaluateJavascript(BRIDGE_SHIM, null)
    }

    override fun onPageFinished(view: WebView?, url: String?) {
        view?.evaluateJavascript(BRIDGE_SHIM, null)
    }

    override fun onReceivedError(view: WebView?, request: WebResourceRequest?, error: WebResourceError?) {
        if (request?.isForMainFrame != true) return
        Log.e(TAG, "Load ${request.url}: ${error?.errorCode} ${error?.description}")
    }

    override fun onReceivedHttpError(view: WebView?, request: WebResourceRequest?, response: WebResourceResponse?) {
        if (request?.isForMainFrame != true) return
        Log.e(TAG, "Load ${request.url}: HTTP ${response?.statusCode}")
    }
}

private class LoggingWebChromeClient : WebChromeClient() {
    override fun onConsoleMessage(message: ConsoleMessage?): Boolean {
        if (message?.messageLevel() == ConsoleMessage.MessageLevel.ERROR) {
            Log.e(TAG, "Console ${message.sourceId()}:${message.lineNumber()} ${message.message()}")
        }
        return false
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
