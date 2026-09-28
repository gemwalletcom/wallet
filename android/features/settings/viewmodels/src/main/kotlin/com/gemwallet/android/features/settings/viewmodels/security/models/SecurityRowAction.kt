package com.gemwallet.android.features.settings.viewmodels.security.models

import uniffi.gemstone.GemRowAction

sealed interface SecurityRowAction {
    data object Authentication : SecurityRowAction
    data object HideBalance : SecurityRowAction
}

fun GemRowAction.securityAction(): SecurityRowAction? = when (this) {
    GemRowAction.Authentication -> SecurityRowAction.Authentication
    GemRowAction.HideBalance -> SecurityRowAction.HideBalance
    else -> null
}
