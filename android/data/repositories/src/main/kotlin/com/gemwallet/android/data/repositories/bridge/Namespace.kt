package com.gemwallet.android.data.repositories.bridge

import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toChainType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.WalletConnectionEvents
import com.wallet.core.primitives.WalletConnectionMethods
import uniffi.gemstone.WalletConnect

enum class ChainNamespace(val string: String, val methods: List<WalletConnectionMethods>) {
    Eip155(
        "eip155",
        listOf(
            WalletConnectionMethods.EthChainId,
            WalletConnectionMethods.PersonalSign,
            WalletConnectionMethods.EthSignTypedData,
            WalletConnectionMethods.EthSignTypedDataV4,
            WalletConnectionMethods.EthSignTransaction,
            WalletConnectionMethods.EthSendTransaction,
            WalletConnectionMethods.WalletAddEthereumChain,
            WalletConnectionMethods.WalletSwitchEthereumChain,
            WalletConnectionMethods.EthSendRawTransaction,
        )
    ),
    Solana(
        Chain.Solana.string,
        listOf(
            WalletConnectionMethods.SolanaSignMessage,
            WalletConnectionMethods.SolanaSignTransaction,
            WalletConnectionMethods.SolanaSignAndSendTransaction,
            WalletConnectionMethods.SolanaSignAllTransactions,
        )
    ),
    Sui(
        Chain.Sui.string,
        listOf(
            WalletConnectionMethods.SuiSignPersonalMessage,
            WalletConnectionMethods.SuiSignTransaction,
            WalletConnectionMethods.SuiSignAndExecuteTransaction,
        )
    ),
    Ton(
        Chain.Ton.string,
        listOf(
            WalletConnectionMethods.TonSendMessage,
            WalletConnectionMethods.TonSignData,
        )
    ),
    Tron(
        Chain.Tron.string,
        listOf(
            WalletConnectionMethods.TronSignMessage,
            WalletConnectionMethods.TronSignTransaction,
            WalletConnectionMethods.TronSendTransaction,
        )
    );

    val methodIds: List<String>
        get() = methods.map { it.string }

    val eventIds: List<String>
        get() = when (this) {
            Solana -> emptyList()
            else -> defaultEventIds
        }

    companion object {
        private val defaultEventIds = listOf(
            WalletConnectionEvents.Connect.string,
            WalletConnectionEvents.Disconnect.string,
            WalletConnectionEvents.ChainChanged.string,
            WalletConnectionEvents.AccountsChanged.string,
        )
    }
}

fun Chain.walletConnectNamespace(): ChainNamespace? {
    return when (this.toChainType()) {
        ChainType.Ethereum -> ChainNamespace.Eip155
        ChainType.Solana -> ChainNamespace.Solana
        ChainType.Sui -> ChainNamespace.Sui
        ChainType.Ton -> ChainNamespace.Ton
        ChainType.Tron -> ChainNamespace.Tron
        else -> null
    }
}

fun Chain.walletConnectReference(): String? {
    return WalletConnect().getReference(string)
}

fun Chain.Companion.fromWalletConnectChainId(walletConnectChainId: String?): Chain? {
    val chainId = walletConnectChainId ?: return null
    return WalletConnect().parseChainId(chainId)?.toChain()
}
