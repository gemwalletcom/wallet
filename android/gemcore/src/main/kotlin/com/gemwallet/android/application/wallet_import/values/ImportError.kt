package com.gemwallet.android.application.wallet_import.values

import com.wallet.core.primitives.Wallet

sealed class ImportError(message: String = "") : Exception(message) {

    object InvalidationSecretPhrase : ImportError()

    object InvalidationPrivateKey : ImportError()

    class InvalidWords(val words: List<String>) : ImportError()

    object InvalidAddress : ImportError()

    class CreateError(message: String) : ImportError(message)

    class DuplicatedWallet(val wallet: Wallet) : ImportError("Duplicated wallet")
}
