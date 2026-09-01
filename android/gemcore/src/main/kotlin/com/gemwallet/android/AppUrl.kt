package com.gemwallet.android

import android.net.Uri
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.PublicUrl
import uniffi.gemstone.SocialUrl

object AppUrl {
    private const val UTM_SOURCE = "gemwallet_android"

    fun docs(item: DocsUrl): String = item.url().withUTM()

    fun page(item: PublicUrl): String = item.url().withUTM()

    fun social(item: SocialUrl): String? = item.url()

    private fun String.withUTM(): String = Uri.parse(this)
        .buildUpon()
        .appendQueryParameter("utm_source", UTM_SOURCE)
        .build()
        .toString()
}
