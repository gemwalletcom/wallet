package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

interface WalletIdGenerator {
    fun generateWalletId(type: WalletType, priorityChain: Chain, priorityAddress: String): WalletId {
        require(priorityAddress.isNotEmpty()) { "Account address cannot be empty" }
        val id = when (type) {
            WalletType.Multicoin -> "${type.string}_$priorityAddress"
            WalletType.Single,
            WalletType.PrivateKey,
            WalletType.View -> "${type.string}_${priorityChain.string}_$priorityAddress"
        }
        return WalletId(id)
    }

    fun getPriorityAccount(accounts: List<Account>): Account? {
        require(accounts.isNotEmpty()) { "Accounts list cannot be empty" }
        return accounts.firstOrNull { it.chain == Chain.Ethereum } ?: accounts.firstOrNull()
    }
}
