package com.gemwallet.android

import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.Platform
import uniffi.gemstone.PublicUrl

object AppUrl {
    fun docs(item: DocsUrl): String = item.urlFor(Platform.ANDROID)

    fun page(item: PublicUrl): String = item.urlFor(Platform.ANDROID)

    fun secretSecurity(kind: GemWalletSecretKind): String = docs(
        when (kind) {
            GemWalletSecretKind.PHRASE -> DocsUrl.HowToSecureSecretPhrase
            GemWalletSecretKind.PRIVATE_KEY -> DocsUrl.WhatIsPrivateKey
        },
    )
    val tokenVerification: String by lazy { docs(DocsUrl.TokenVerification) }
}
