package com.gemwallet.android.domains.search

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

fun WalletSearchTag.encode(): String = when (this) {
    WalletSearchTag.All -> "all"
    is WalletSearchTag.Filter -> "filter:${tag.string}"
    is WalletSearchTag.List -> "list:$id"
}

fun walletSearchTagOf(encoded: String?): WalletSearchTag = when {
    encoded == null || encoded == "all" -> WalletSearchTag.All
    encoded.startsWith("filter:") ->
        AssetTag.entries.firstOrNull { it.string == encoded.removePrefix("filter:") }
            ?.let { WalletSearchTag.Filter(it) } ?: WalletSearchTag.All
    encoded.startsWith("list:") -> WalletSearchTag.List(encoded.removePrefix("list:"))
    else -> WalletSearchTag.All
}
