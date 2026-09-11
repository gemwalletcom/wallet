package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletSecretKind

internal fun SavedStateHandle.requireWalletId(): WalletId {
    val value = checkNotNull(get<String>(RouteArgument.WalletId.key)) {
        "Missing route argument: ${RouteArgument.WalletId.key}"
    }
    check(value.isNotBlank()) {
        "Blank route argument: ${RouteArgument.WalletId.key}"
    }
    return WalletId(value)
}

internal fun SavedStateHandle.requireSecretKind(): GemWalletSecretKind =
    checkNotNull(get<GemWalletSecretKind>(RouteArgument.Type.key)) {
        "Missing route argument: ${RouteArgument.Type.key}"
    }

internal fun SavedStateHandle.chain(): Chain? =
    get<String>(RouteArgument.Chain.key)?.let { value -> Chain.entries.firstOrNull { it.string == value } }
