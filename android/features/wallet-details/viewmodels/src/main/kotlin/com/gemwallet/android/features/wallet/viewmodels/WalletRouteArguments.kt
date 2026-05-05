package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

internal fun SavedStateHandle.requireWalletId(): WalletId {
    val value = checkNotNull(get<String>(RouteArgument.WalletId.key)) {
        "Missing route argument: ${RouteArgument.WalletId.key}"
    }
    check(value.isNotBlank()) {
        "Blank route argument: ${RouteArgument.WalletId.key}"
    }
    return WalletId(value)
}

internal fun SavedStateHandle.requireWalletType(): WalletType =
    checkNotNull(get<WalletType>(RouteArgument.Type.key)) {
        "Missing route argument: ${RouteArgument.Type.key}"
    }
