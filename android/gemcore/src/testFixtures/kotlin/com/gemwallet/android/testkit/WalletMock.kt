package com.gemwallet.android.testkit

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType

fun mockWalletId(id: String = "wallet-1") = WalletId(id)

fun mockWallet(
    id: String = "wallet-1",
    name: String = "Wallet",
    index: Int = 0,
    type: WalletType = WalletType.Multicoin,
    accounts: List<Account> = emptyList(),
    order: Int = 0,
    isPinned: Boolean = false,
    source: WalletSource = WalletSource.Create,
) = Wallet(
    id = WalletId(id),
    name = name,
    index = index,
    type = type,
    accounts = accounts,
    order = order,
    isPinned = isPinned,
    source = source,
)
