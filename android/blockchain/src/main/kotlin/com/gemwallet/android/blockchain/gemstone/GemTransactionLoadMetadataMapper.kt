package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.blockchain.clients.algorand.toChainData
import com.gemwallet.android.blockchain.clients.aptos.toChainData
import com.gemwallet.android.blockchain.clients.bitcoin.toChainData
import com.gemwallet.android.blockchain.clients.cardano.toChainData
import com.gemwallet.android.blockchain.clients.cosmos.toChainData
import com.gemwallet.android.blockchain.clients.ethereum.toChainData
import com.gemwallet.android.blockchain.clients.hyper.toChainData
import com.gemwallet.android.blockchain.clients.near.toChainData
import com.gemwallet.android.blockchain.clients.polkadot.toChainData
import com.gemwallet.android.blockchain.clients.solana.toChainData
import com.gemwallet.android.blockchain.clients.stellar.toChainData
import com.gemwallet.android.blockchain.clients.sui.toChainData
import com.gemwallet.android.blockchain.clients.ton.toChainData
import com.gemwallet.android.blockchain.clients.tron.toChainData
import com.gemwallet.android.blockchain.clients.xrp.toChainData
import com.gemwallet.android.model.ChainSignData
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.SwapperException.NotSupportedChain

internal fun GemTransactionLoadMetadata.toChainData(): ChainSignData = when (this) {
    is GemTransactionLoadMetadata.Algorand -> toChainData()
    is GemTransactionLoadMetadata.Aptos -> toChainData()
    is GemTransactionLoadMetadata.Bitcoin -> toChainData()
    is GemTransactionLoadMetadata.Zcash -> toChainData()
    is GemTransactionLoadMetadata.Cardano -> toChainData()
    is GemTransactionLoadMetadata.Cosmos -> toChainData()
    is GemTransactionLoadMetadata.Evm -> toChainData()
    is GemTransactionLoadMetadata.Near -> toChainData()
    is GemTransactionLoadMetadata.Polkadot -> toChainData()
    is GemTransactionLoadMetadata.Solana -> toChainData()
    is GemTransactionLoadMetadata.Stellar -> toChainData()
    is GemTransactionLoadMetadata.Sui -> toChainData()
    is GemTransactionLoadMetadata.Ton -> toChainData()
    is GemTransactionLoadMetadata.Tron -> toChainData()
    is GemTransactionLoadMetadata.Xrp -> toChainData()
    is GemTransactionLoadMetadata.Hyperliquid -> toChainData()
    GemTransactionLoadMetadata.None -> throw NotSupportedChain()
}
