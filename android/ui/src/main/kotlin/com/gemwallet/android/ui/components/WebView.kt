package com.gemwallet.android.ui.components

import android.annotation.SuppressLint
import android.content.Intent
import android.net.Uri
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
                    val client = HostWebViewClient(url.toUri().host.orEmpty())
                    webViewClient = client
                    bridge?.let { addJavascriptInterface(it, it.name) }
                    if (client.isAllowed(url.toUri())) loadUrl(url) else client.openExternally(this, url.toUri())
                }
            },
        )
    }
}

private class HostWebViewClient(private val host: String) : WebViewClient() {
    override fun shouldOverrideUrlLoading(view: AndroidWebView, request: WebResourceRequest): Boolean {
        if (request.url.scheme !in webSchemes) return true
        if (!request.isForMainFrame) return false
        if (isAllowed(request.url)) return false
        openExternally(view, request.url)
        return true
    }

    fun isAllowed(url: Uri): Boolean {
        val target = url.host.orEmpty()
        return url.scheme == "https" && (target == host || target.endsWith(".$host"))
    }

    fun openExternally(view: AndroidWebView, url: Uri) {
        runCatching { view.context.startActivity(Intent(Intent.ACTION_VIEW, url)) }
    }

    private companion object {
        val webSchemes = setOf("http", "https")
    }
}
