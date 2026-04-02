package com.gemwallet.android.features.settings.networks.viewmodels

import java.net.URI

internal fun String.isValidNodeUrl(): Boolean {
    val uri = runCatching { URI(this) }.getOrNull() ?: return false
    val scheme = uri.scheme?.lowercase()
    return scheme in setOf("http", "https") && !uri.host.isNullOrBlank()
}
