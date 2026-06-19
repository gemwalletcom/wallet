package com.gemwallet.android.domains.wallet.values

interface WalletSecretDataValue {
    val data: List<String>

    val isError: Boolean

    fun phrase(): List<String>

    fun privateKey(): String?
}