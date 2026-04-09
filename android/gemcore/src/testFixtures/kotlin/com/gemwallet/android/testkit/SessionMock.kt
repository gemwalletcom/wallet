package com.gemwallet.android.testkit

import com.gemwallet.android.model.Session
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet

fun mockSession(
    wallet: Wallet = mockWallet(),
    currency: Currency = Currency.USD,
) = Session(
    wallet = wallet,
    currency = currency,
)
