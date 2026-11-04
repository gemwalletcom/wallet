package com.gemwallet.android.features.onboarding.presents.create_wallet

internal sealed interface CreateWalletAction {
    data object Copy : CreateWalletAction
    data object Continue : CreateWalletAction
    data object Cancel : CreateWalletAction
}
