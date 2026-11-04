package com.gemwallet.android.features.settings.presents.chain_settings

internal sealed interface AddNodeAction {
    data object UrlChange : AddNodeAction
    data object Add : AddNodeAction
    data object Cancel : AddNodeAction
}
