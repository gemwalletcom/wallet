package com.gemwallet.android.ui.components

import android.annotation.SuppressLint
import android.content.Intent
import android.webkit.WebResourceRequest
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.net.toUri
import android.webkit.WebView as AndroidWebView

interface WebViewBridge {
    val name: String
}

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun WebView(url: String, bridge: WebViewBridge? = null, modifier: Modifier = Modifier) {
    key(url) {
        AndroidView(
            modifier = modifier,
            factory = { context ->
                AndroidWebView(context).apply {
                    settings.javaScriptEnabled = true
                    settings.domStorageEnabled = true
                    webViewClient = HostWebViewClient(url.toUri().host.orEmpty())
                    bridge?.let { addJavascriptInterface(it, it.name) }
                    loadUrl(url)
                }
            },
        )
    }
}

private class HostWebViewClient(private val host: String) : WebViewClient() {
    override fun shouldOverrideUrlLoading(view: AndroidWebView, request: WebResourceRequest): Boolean {
        if (request.url.scheme !in webSchemes) return true
        if (!request.isForMainFrame) return false
        val target = request.url.host.orEmpty()
        if (request.url.scheme == "https" && (target == host || target.endsWith(".$host"))) return false
        runCatching { view.context.startActivity(Intent(Intent.ACTION_VIEW, request.url)) }
        return true
    }

    private companion object {
        val webSchemes = setOf("http", "https")
    }
}
