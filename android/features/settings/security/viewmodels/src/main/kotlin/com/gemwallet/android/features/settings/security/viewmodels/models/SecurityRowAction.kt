package com.gemwallet.android.features.settings.security.viewmodels.models

import uniffi.gemstone.GemListRowTitle

sealed interface SecurityRowAction {
    data object Authentication : SecurityRowAction
    data object HideBalance : SecurityRowAction
}

fun GemListRowTitle.securityAction(): SecurityRowAction? = when (this) {
    GemListRowTitle.AUTHENTICATION -> SecurityRowAction.Authentication
    GemListRowTitle.HIDE_BALANCE -> SecurityRowAction.HideBalance
    else -> null
}
