package com.gemwallet.android.features.onboarding.presents.import_wallet

import com.gemwallet.android.model.ImportType

internal sealed interface ImportWalletAction {
    data object Import : ImportWalletAction
    data object Scan : ImportWalletAction
    data class SelectType(val type: ImportType) : ImportWalletAction
    data class SelectSuggestion(val word: String) : ImportWalletAction
    data object Cancel : ImportWalletAction
}
