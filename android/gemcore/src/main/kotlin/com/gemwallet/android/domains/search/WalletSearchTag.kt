package com.gemwallet.android.domains.search

import com.gemwallet.android.serializer.decodeJsonOrNull
import com.gemwallet.android.serializer.toJson
import kotlinx.serialization.Serializable

@Serializable
sealed interface WalletSearchTag {
    @Serializable
    data object All : WalletSearchTag

    @Serializable
    data class List(val id: String) : WalletSearchTag
}

fun WalletSearchTag.encode(): String = toJson()

fun walletSearchTagOf(encoded: String?): WalletSearchTag = encoded.decodeJsonOrNull() ?: WalletSearchTag.All
