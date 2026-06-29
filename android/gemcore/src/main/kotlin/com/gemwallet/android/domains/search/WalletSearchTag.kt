package com.gemwallet.android.domains.search

import com.wallet.core.primitives.AssetTag

sealed interface WalletSearchTag {
    data object All : WalletSearchTag
    data class Filter(val tag: AssetTag) : WalletSearchTag
    data class List(val id: String) : WalletSearchTag
}

val WalletSearchTag.apiTag: String?
    get() = when (this) {
        WalletSearchTag.All -> null
        is WalletSearchTag.Filter -> tag.string
        is WalletSearchTag.List -> id
    }

val WalletSearchTag.includesPerpetuals: Boolean
    get() = this !is WalletSearchTag.Filter

val WalletSearchTag.isAll: Boolean
    get() = this is WalletSearchTag.All
