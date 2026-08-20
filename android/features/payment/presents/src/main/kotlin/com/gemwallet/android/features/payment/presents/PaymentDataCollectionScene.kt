package com.gemwallet.android.features.payment.presents

import android.annotation.SuppressLint
import android.content.Context
import android.view.ContextThemeWrapper
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
import androidx.compose.foundation.layout.fillMaxHeight
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.open
import org.json.JSONObject
import androidx.core.net.toUri

private const val MESSAGE_HANDLER = "payDataCollectionComplete"
private const val MESSAGE_TYPE_KEY = "type"
private const val MESSAGE_ERROR_KEY = "error"
private const val COMPLETE = "IC_COMPLETE"
private const val ERROR = "IC_ERROR"
private const val TAG = "PaymentDataCollection"

@SuppressLint("SetJavaScriptEnabled")
@Composable
internal fun PaymentDataCollectionModal(
    url: String?,
    onAction: (PaymentSceneAction) -> Unit,
) {
    var webView by remember { mutableStateOf<WebView?>(null) }
    val uriHandler = LocalUriHandler.current

    if (url != null) {
        BackHandler {
            val view = webView
            if (view != null && view.canGoBack()) view.goBack() else onAction(PaymentSceneAction.DismissDataCollection)
        }
    }

    ModalBottomSheet(
        isVisible = url != null,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.transfer_payment_title),
        onDismissRequest = { onAction(PaymentSceneAction.DismissDataCollection) },
    ) {
        url ?: return@ModalBottomSheet
        AndroidView(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(),
            factory = { context ->
                if (context.applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0) {
                    WebView.setWebContentsDebuggingEnabled(true)
                }
                WebView(ContextThemeWrapper(context, context.applicationInfo.theme)).apply {
                    layoutParams = ViewGroup.LayoutParams(MATCH_PARENT, MATCH_PARENT)
                    settings.javaScriptEnabled = true
                    settings.domStorageEnabled = true
                    settings.useWideViewPort = true
                    settings.loadWithOverviewMode = true
                    CookieManager.getInstance().setAcceptThirdPartyCookies(this, true)
                    webViewClient = AllowedHostWebViewClient(context, uriHandler, url.toUri().host.orEmpty())
                    webChromeClient = LoggingWebChromeClient()
                    addJavascriptInterface(CollectDataBridge(onAction), MESSAGE_HANDLER)
                    loadUrl(url)
                    webView = this
                }
            },
            onRelease = { view ->
                view.removeJavascriptInterface(MESSAGE_HANDLER)
                view.destroy()
                webView = null
            },
        )
    }
}

private class CollectDataBridge(
    private val onAction: (PaymentSceneAction) -> Unit,
) {
    @JavascriptInterface
    fun postMessage(payload: String) {
        val message = runCatching { JSONObject(payload) }.getOrNull() ?: return
        when (message.optString(MESSAGE_TYPE_KEY)) {
            COMPLETE -> onAction(PaymentSceneAction.DataCollected)
            ERROR -> onAction(PaymentSceneAction.DataCollectionFailed(message.optString(MESSAGE_ERROR_KEY).takeIf { it.isNotEmpty() }))
        }
    }
}

private class AllowedHostWebViewClient(
    private val context: Context,
    private val uriHandler: UriHandler,
    private val allowedHost: String,
) : WebViewClient() {

    override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
        val uri = request?.url ?: return true
        if (uri.scheme != "https" && uri.scheme != "http") return true
        val host = uri.host?.lowercase() ?: return true
        if (uri.scheme == "https" && (host == allowedHost || host.endsWith(".$allowedHost"))) return false
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
