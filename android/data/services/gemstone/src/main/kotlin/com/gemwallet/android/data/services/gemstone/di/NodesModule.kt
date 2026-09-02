package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.wallet.core.primitives.Chain
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.Config
import uniffi.gemstone.GemNodeService

@InstallIn(SingletonComponent::class)
@Module
object NodesModule {

    @Provides
    @Singleton
    fun provideGemstoneConfig(): Config = Config()

    @Provides
    @Singleton
    fun provideNodeService(
        nodesDao: NodesDao,
        preferences: GemstonePreferencesStore,
    ): GemNodeService = GemNodeService(nodesDao, preferences)

    @Provides
    fun provideGetNodeUrlCase(
        nodeService: GemNodeService,
    ): GetNodeUrlCase = object : GetNodeUrlCase {
        override fun getNodeUrl(chain: Chain) = nodeService.nodeUrl(chain.string)

        override fun getWebSocketNodeUrl(chain: Chain) = nodeService.websocketNodeUrl(chain.string)
    }
}
