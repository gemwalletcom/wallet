package com.gemwallet.android.features.settings.viewmodels.security.models

import uniffi.gemstone.GemRowTap

sealed interface SecurityRowAction {
    data object Authentication : SecurityRowAction
    data object HideBalance : SecurityRowAction
}

fun GemRowTap.securityAction(): SecurityRowAction? = when (this) {
    GemRowTap.Authentication -> SecurityRowAction.Authentication
    GemRowTap.HideBalance -> SecurityRowAction.HideBalance
    else -> null
}
