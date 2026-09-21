package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSession
import uniffi.gemstone.GemWalletConnectMessageRequest
import uniffi.gemstone.SignMessage

fun mockGemWalletConnectMessageRequest(wallet: Wallet = mockWalletMulticoin(), session: WalletConnectionSession = mockWalletConnectionSession(), message: SignMessage = mockSignMessage()) = GemWalletConnectMessageRequest(
    sessionId = session.sessionId,
    chain = Chain.Ethereum.string,
    wallet = wallet.toGem(),
    account = wallet.accounts.first().toGem(),
    session = session.toGem(),
    simulation = mockSimulationResult(),
    message = message,
    assets = emptyList(),
)
