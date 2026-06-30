package com.gemwallet.android.domains.search

import com.gemwallet.android.serializer.decodeArg
import com.gemwallet.android.serializer.encodeArg
import com.wallet.core.primitives.AssetTag
import kotlinx.serialization.Serializable

@Serializable
sealed interface WalletSearchTag {
    @Serializable
    data object All : WalletSearchTag

    @Serializable
    data class Filter(val tag: AssetTag) : WalletSearchTag

    @Serializable
    data class List(val id: String) : WalletSearchTag
}

val WalletSearchTag.apiTag: String?
    get() = when (this) {
        WalletSearchTag.All -> null
        is WalletSearchTag.Filter -> tag.string
        is WalletSearchTag.List -> id
    }

val WalletSearchTag.includesPerpetuals: Boolean
    get() = when (this) {
        is WalletSearchTag.Filter -> false
        WalletSearchTag.All, is WalletSearchTag.List -> true
    }

val WalletSearchTag.isAll: Boolean
    get() = this is WalletSearchTag.All

fun AssetTag?.toWalletSearchTag(): WalletSearchTag =
    this?.let { WalletSearchTag.Filter(it) } ?: WalletSearchTag.All

fun WalletSearchTag.encode(): String = encodeArg()

fun walletSearchTagOf(encoded: String?): WalletSearchTag = encoded.decodeArg() ?: WalletSearchTag.All
