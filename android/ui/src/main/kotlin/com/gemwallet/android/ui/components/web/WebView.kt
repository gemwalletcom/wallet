package com.gemwallet.android.ui.components.web

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Bitmap
import android.view.ContextThemeWrapper
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams.MATCH_PARENT
import android.webkit.CookieManager
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import com.gemwallet.android.ui.open
import org.json.JSONObject
import android.webkit.WebView as AndroidWebView

class WebViewMessageHandler(
    val name: String,
    val script: String? = null,
    val onMessage: (JSONObject) -> Unit,
)

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun WebView(
    url: String,
    modifier: Modifier = Modifier,
    messageHandler: WebViewMessageHandler? = null,
    onBack: (() -> Unit)? = null,
) {
    val uriHandler = LocalUriHandler.current
    var webView by remember { mutableStateOf<AndroidWebView?>(null) }

    if (onBack != null) {
        BackHandler {
            val view = webView
            if (view != null && view.canGoBack()) view.goBack() else onBack()
        }
    }

    AndroidView(
        modifier = modifier,
        factory = { context ->
            AndroidWebView(ContextThemeWrapper(context, context.applicationInfo.theme)).apply {
                layoutParams = ViewGroup.LayoutParams(MATCH_PARENT, MATCH_PARENT)
                settings.javaScriptEnabled = true
                settings.domStorageEnabled = true
                settings.useWideViewPort = true
                settings.loadWithOverviewMode = true
                CookieManager.getInstance().setAcceptThirdPartyCookies(this, true)
                webViewClient = AllowedHostWebViewClient(
                    context = context,
                    uriHandler = uriHandler,
                    allowedHost = url.toUri().host.orEmpty(),
                    script = messageHandler?.script,
                )
                messageHandler?.let { addJavascriptInterface(MessageBridge(it), it.name) }
                loadUrl(url)
                webView = this
            }
        },
        onRelease = { view ->
            messageHandler?.let { view.removeJavascriptInterface(it.name) }
            view.destroy()
            webView = null
        },
    )
}

private class MessageBridge(private val handler: WebViewMessageHandler) {
    @JavascriptInterface
    fun postMessage(payload: String) {
        val message = runCatching { JSONObject(payload) }.getOrNull() ?: return
        handler.onMessage(message)
    }
}

private class AllowedHostWebViewClient(
    private val context: Context,
    private val uriHandler: UriHandler,
    private val allowedHost: String,
    private val script: String?,
) : WebViewClient() {

    override fun shouldOverrideUrlLoading(view: AndroidWebView?, request: WebResourceRequest?): Boolean {
        val uri = request?.url ?: return true
        if (uri.scheme != "https" && uri.scheme != "http") return true
        val host = uri.host?.lowercase() ?: return true
        if (uri.scheme == "https" && (host == allowedHost || host.endsWith(".$allowedHost"))) return false
        uriHandler.open(context, uri.toString())
        return true
    }

    override fun onPageStarted(view: AndroidWebView?, url: String?, favicon: Bitmap?) {
        script?.let { view?.evaluateJavascript(it, null) }
    }

    override fun onPageFinished(view: AndroidWebView?, url: String?) {
        script?.let { view?.evaluateJavascript(it, null) }
    }
}
