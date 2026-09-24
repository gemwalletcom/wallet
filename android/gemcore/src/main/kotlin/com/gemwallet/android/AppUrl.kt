package com.gemwallet.android

import uniffi.gemstone.DocsUrl
import uniffi.gemstone.Platform
import uniffi.gemstone.PublicUrl

object AppUrl {
    fun docs(item: DocsUrl): String = item.urlFor(Platform.ANDROID)

    fun page(item: PublicUrl): String = item.urlFor(Platform.ANDROID)

    fun staking(chain: String): String = docs(DocsUrl.Staking(chain))

    val howToSecureSecretPhrase: String by lazy { docs(DocsUrl.HowToSecureSecretPhrase) }
    val networkFees: String by lazy { docs(DocsUrl.NetworkFees) }
    val tokenVerification: String by lazy { docs(DocsUrl.TokenVerification) }
}
