package com.gemwallet.android.application.wallet_connect.cases

import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectAuthMessage
import uniffi.gemstone.GemWalletConnectAuthentication
import uniffi.gemstone.GemWalletConnectRejectionReason

interface ApproveWalletConnectAuthentication {
    fun authMessage(authentication: GemWalletConnectAuthentication, wallet: Wallet): GemWalletConnectAuthMessage

    fun approveAuthentication(authentication: GemWalletConnectAuthentication, wallet: Wallet, signature: String, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit)

    fun rejectAuthentication(authentication: GemWalletConnectAuthentication, reason: GemWalletConnectRejectionReason = GemWalletConnectRejectionReason.USER_REJECTED, onSuccess: () -> Unit = {}, onError: (GemErrorText) -> Unit = {})
}
