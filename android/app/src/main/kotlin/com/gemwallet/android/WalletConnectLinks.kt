package com.gemwallet.android

import java.net.URI

internal sealed interface WalletConnectLink {
    data class Pairing(val uri: String) : WalletConnectLink
    data object Request : WalletConnectLink
    data object Session : WalletConnectLink
}

internal fun String.toWalletConnectLink(): WalletConnectLink? {
    val uri = runCatching { URI(this) }.getOrNull() ?: return null
    val queryParameters = uri.walletConnectQueryParameters()
    return when {
        WalletConnectScheme.equals(uri.scheme, ignoreCase = true) -> {
            // Keep this parser classification-only; Reown validates direct wc: pairing payloads.
            queryParameters.toWalletConnectCallback() ?: WalletConnectLink.Pairing(this)
        }
        GemScheme.equals(uri.scheme, ignoreCase = true) && WalletConnectHost.equals(uri.host, ignoreCase = true) -> {
            toGemWalletConnectLink(queryParameters)
        }
        else -> null
    }
}

private fun toGemWalletConnectLink(queryParameters: Map<String, String>): WalletConnectLink? {
    return WalletConnectQuery.Uri.nonBlankValue(queryParameters)?.let(WalletConnectLink::Pairing)
        ?: queryParameters.toWalletConnectCallback()
}

private fun Map<String, String>.toWalletConnectCallback(): WalletConnectLink? {
    if (WalletConnectQuery.SessionTopic.nonBlankValue(this) != null) return WalletConnectLink.Session
    return WalletConnectLink.Request.takeIf { WalletConnectQuery.RequestId.existsIn(this) }
}

private fun URI.walletConnectQueryParameters(): Map<String, String> {
    val query = rawQuery ?: rawSchemeSpecificPart
        ?.substringAfter('?', missingDelimiterValue = "")
        ?.takeIf { it.isNotBlank() }
    return query.decodeUriQueryParameters()
}

private enum class WalletConnectQuery(val value: String) {
    RequestId("requestId"),
    SessionTopic("sessionTopic"),
    Uri("uri");

    fun nonBlankValue(queryParameters: Map<String, String>): String? {
        return queryParameters[value]?.takeIf { it.isNotBlank() }
    }

    fun existsIn(queryParameters: Map<String, String>): Boolean {
        return value in queryParameters
    }
}

private const val WalletConnectScheme = "wc"
private const val GemScheme = "gem"
private const val WalletConnectHost = "wc"
