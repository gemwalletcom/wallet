package com.gemwallet.android.features.settings.networks.viewmodels

import java.net.URI

internal object NodeUrlParser {
    fun parse(input: String): String? {
        if (input.isEmpty()) {
            return null
        }

        val hasExplicitScheme = "://" in input
        val candidateUrl = if (hasExplicitScheme) input else NodeUrlScheme.Https.url(input)
        val uri = runCatching { URI(candidateUrl) }.getOrNull()
        val scheme = NodeUrlScheme.from(uri?.scheme)
        val host = uri?.host

        return candidateUrl.takeIf {
            scheme != null &&
                !host.isNullOrBlank() &&
                (hasExplicitScheme || '.' in host)
        }
    }
}

private enum class NodeUrlScheme(val value: String) {
    Http("http"),
    Https("https");

    fun url(host: String): String = "$value://$host"

    companion object {
        fun from(value: String?): NodeUrlScheme? = entries.firstOrNull {
            it.value.equals(value, ignoreCase = true)
        }
    }
}
