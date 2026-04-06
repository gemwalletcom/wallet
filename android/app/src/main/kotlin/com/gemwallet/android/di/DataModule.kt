package com.gemwallet.android.di

import com.gemwallet.android.blockchain.clients.algorand.AlgorandSignClient
import com.gemwallet.android.blockchain.clients.bitcoin.BitcoinSignClient
import com.gemwallet.android.blockchain.clients.cardano.CardanoSignClient
import com.gemwallet.android.blockchain.clients.cosmos.CosmosSignClient
import com.gemwallet.android.blockchain.clients.near.NearSignClient
import com.gemwallet.android.blockchain.clients.polkadot.PolkadotSignClient
import com.gemwallet.android.blockchain.clients.solana.SolanaSignClient
import com.gemwallet.android.blockchain.clients.stellar.StellarSignClient
import com.gemwallet.android.blockchain.clients.sui.SuiSignClient
import com.gemwallet.android.blockchain.clients.ton.TonSignClient
import com.gemwallet.android.blockchain.clients.tron.TronSignClient
import com.gemwallet.android.blockchain.clients.xrp.XrpSignClient
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.blockchain.services.SignClientProxy
import com.gemwallet.android.blockchain.services.SignService
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.services.SyncService
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainType
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DataModule {

    @Provides
    @Singleton
    fun providesBroadcastProxy(
        gateway: GemGateway,
    ): BroadcastService = BroadcastService(
        gateway = gateway,
    )

    @Provides
    @Singleton
    fun provideSignerPreloader(
        gateway: GemGateway,
    ): SignerPreloaderProxy {
        return SignerPreloaderProxy(
            gateway = gateway,
        )
    }

    @Provides
    @Singleton
    fun provideSignService(
        assetsRepository: AssetsRepository,
    ): SignClientProxy = SignClientProxy(
        clients = Chain.available().mapNotNull {
            when (it.toChainType()) {
                ChainType.Bitcoin -> BitcoinSignClient(it)
                ChainType.Solana -> SolanaSignClient(it, assetsRepository)
                ChainType.Cosmos -> CosmosSignClient(it)
                ChainType.Ton -> TonSignClient(it)
                ChainType.Tron -> TronSignClient(it)

                ChainType.Xrp -> XrpSignClient(it)
                ChainType.Near -> NearSignClient(it)
                ChainType.Algorand -> AlgorandSignClient(it)
                ChainType.Stellar -> StellarSignClient(it)
                ChainType.Polkadot -> PolkadotSignClient(it)
                ChainType.Cardano -> CardanoSignClient(it)
                ChainType.Ethereum,
                ChainType.Aptos,
                ChainType.Sui,
                ChainType.HyperCore -> return@mapNotNull null
            }
        } + listOf(SignService()),
    )

    @Singleton
    @Provides
    fun provideNodeStatusService(
        gateway: GemGateway,
    ): NodeStatusService {
        return NodeStatusService(gateway)
    }

    @Singleton
    @Provides
    fun provideSyncService(
        buyRepository: BuyRepository,
        syncDeviceInfo: SyncDeviceInfo,
    ): SyncService {
        return SyncService(
            buyRepository = buyRepository,
            syncDeviceInfo = syncDeviceInfo,
        )
    }
}
