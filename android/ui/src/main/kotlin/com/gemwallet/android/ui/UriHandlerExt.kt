package com.gemwallet.android.ui

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.LocaleList
import android.provider.Browser
import androidx.compose.ui.platform.UriHandler
import androidx.core.net.toUri

private const val CUSTOM_TABS_SESSION = "android.support.customtabs.extra.SESSION"
private const val CUSTOM_TABS_INSTANT_APPS = "android.support.customtabs.extra.EXTRA_ENABLE_INSTANT_APPS"
private const val CUSTOM_TABS_SHARE_STATE = "androidx.browser.customtabs.extra.SHARE_STATE"
private const val ACCEPT_LANGUAGE = "Accept-Language"

fun UriHandler.open(context: Context, uri: String) {
    try {
        context.startActivity(customTabIntent(uri))
    } catch (_: Throwable) {
        try {
            openUri(uri)
        } catch (_: Throwable) {
        }
    }
}

private fun customTabIntent(uri: String): Intent {
    val intent = Intent(Intent.ACTION_VIEW, uri.toUri())
        .putExtras(Bundle().apply { putBinder(CUSTOM_TABS_SESSION, null) })
        .putExtra(CUSTOM_TABS_INSTANT_APPS, true)
        .putExtra(CUSTOM_TABS_SHARE_STATE, 0)
    val locales = LocaleList.getAdjustedDefault()
    if (!locales.isEmpty) {
        intent.putExtra(Browser.EXTRA_HEADERS, Bundle().apply { putString(ACCEPT_LANGUAGE, locales[0].toLanguageTag()) })
    }
    return intent
}
