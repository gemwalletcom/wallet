package com.gemwallet.android

import android.net.Uri
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.PublicUrl

object AppUrl {
    private const val UTM_SOURCE = "gemwallet_android"

    fun docs(item: DocsUrl): String = item.url().withUTM()

    fun page(item: PublicUrl): String = item.url().withUTM()

    fun staking(chain: String): String = docs(DocsUrl.Staking(chain))

    val howToSecureSecretPhrase: String by lazy { docs(DocsUrl.HowToSecureSecretPhrase) }
    val networkFees: String by lazy { docs(DocsUrl.NetworkFees) }
    val tokenVerification: String by lazy { docs(DocsUrl.TokenVerification) }

    private fun String.withUTM(): String = Uri.parse(this)
        .buildUpon()
        .appendQueryParameter("utm_source", UTM_SOURCE)
        .build()
        .toString()
}
