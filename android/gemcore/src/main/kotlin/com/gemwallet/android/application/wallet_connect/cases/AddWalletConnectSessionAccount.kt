package com.gemwallet.android.application.wallet_connect.cases

interface AddWalletConnectSessionAccount {
    fun addSessionAccount(topic: String, account: String, onComplete: () -> Unit)
}
