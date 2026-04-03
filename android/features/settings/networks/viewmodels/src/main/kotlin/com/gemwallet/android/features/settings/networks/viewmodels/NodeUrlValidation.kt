package com.gemwallet.android.features.settings.networks.viewmodels

import java.net.URI

internal object NodeUrlParser {
    fun parse(input: String): String? {
        if (input.isEmpty()) {
            return null
        }

        val hasExplicitScheme = "://" in input
        val candidateUrl = if (hasExplicitScheme) input else "https://$input"
        val uri = runCatching { URI(candidateUrl) }.getOrNull()
        val scheme = uri?.scheme?.lowercase()
        val host = uri?.host

        return candidateUrl.takeIf {
            scheme in setOf("http", "https") &&
                !host.isNullOrBlank() &&
                (hasExplicitScheme || '.' in host)
        }
    }
}
