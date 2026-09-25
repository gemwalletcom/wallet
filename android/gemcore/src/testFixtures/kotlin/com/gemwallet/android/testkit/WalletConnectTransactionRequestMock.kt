package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSession
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemWalletConnectTransactionAction
import uniffi.gemstone.GemWalletConnectTransactionRequest

fun mockGemWalletConnectTransactionRequest(
    transfer: GemTransferData,
    action: GemWalletConnectTransactionAction,
    wallet: Wallet = mockWallet(id = mockWalletId(address = "0xabc"), name = "Main Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc"))),
    session: WalletConnectionSession = mockWalletConnectionSession(),
) = GemWalletConnectTransactionRequest(
    sessionId = session.sessionId,
    chain = Chain.Ethereum.string,
    wallet = wallet.toGem(),
    account = wallet.accounts.first().toGem(),
    session = session.toGem(),
    simulation = mockSimulationResult(),
    transfer = transfer,
    action = action,
)
