package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnectionMethods
import uniffi.gemstone.GemChainServiceInterface

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
            WalletConnectionMethods.SuiGetAccounts,
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
}

fun Chain.Companion.fromWalletConnectChainId(chainService: GemChainServiceInterface, walletConnectChainId: String?): Chain? {
    val chainId = walletConnectChainId ?: return null
    return chainService.chainFromCaip2(chainId)?.requireChain()
}
