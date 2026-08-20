package com.gemwallet.android.domains.search

import com.gemwallet.android.serializer.fromJson
import com.gemwallet.android.serializer.toJson
import kotlinx.serialization.Serializable

@Serializable
sealed interface WalletSearchTag {
    @Serializable
    data object All : WalletSearchTag

    @Serializable
    data class List(val id: String) : WalletSearchTag
}

val WalletSearchTag.apiTag: String?
    get() = when (this) {
        WalletSearchTag.All -> null
        is WalletSearchTag.List -> id
    }

val WalletSearchTag.isAll: Boolean
    get() = this is WalletSearchTag.All

fun WalletSearchTag.encode(): String = toJson()

fun walletSearchTagOf(encoded: String?): WalletSearchTag = encoded.fromJson() ?: WalletSearchTag.All
