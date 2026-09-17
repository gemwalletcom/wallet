package com.gemwallet.android.ui.components

import android.annotation.SuppressLint
import android.webkit.WebResourceRequest
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import android.webkit.WebView as AndroidWebView

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun WebView(
    url: String,
    javascriptInterface: Pair<String, Any>? = null,
    modifier: Modifier = Modifier,
) {
    key(url) {
        AndroidView(
            modifier = modifier,
            factory = { context ->
                AndroidWebView(context).apply {
                    settings.javaScriptEnabled = true
                    settings.domStorageEnabled = true
                    webViewClient = HostWebViewClient(url.toUri().host.orEmpty())
                    javascriptInterface?.let { (name, bridge) -> addJavascriptInterface(bridge, name) }
                    loadUrl(url)
                }
            },
        )
    }
}

private class HostWebViewClient(private val host: String) : WebViewClient() {
    override fun shouldOverrideUrlLoading(view: AndroidWebView, request: WebResourceRequest): Boolean {
        val target = request.url.host.orEmpty()
        return !(target == host || target.endsWith(".$host"))
    }
}
