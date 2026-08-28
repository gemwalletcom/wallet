package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.data.repositories.gemstone.GemstoneNodeStore
import com.gemwallet.android.data.repositories.nodes.NodesRepository
import com.gemwallet.android.data.service.store.ConfigStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Named
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
    @Named("node")
    fun provideNodeConfigStore(@ApplicationContext context: Context): ConfigStore = ConfigStore(
        context.getSharedPreferences("node-config", Context.MODE_PRIVATE)
    )

    @Provides
    @Singleton
    fun provideNodeStore(
        @Named("node") configStore: ConfigStore,
    ): GemstoneNodeStore = GemstoneNodeStore(configStore)

    @Provides
    @Singleton
    fun provideNodeService(nodeStore: GemstoneNodeStore): GemNodeService = GemNodeService(nodeStore)

    @Provides
    @Singleton
    fun provideNodesRepository(
        nodeService: GemNodeService,
        nodeStore: GemstoneNodeStore,
    ): NodesRepository = NodesRepository(nodeService = nodeService, nodeStore = nodeStore)

    @Provides
    fun provideSetCurrentNodeCase(repository: NodesRepository): SetCurrentNodeCase = repository

    @Provides
    fun provideGetCurrentNodeCase(repository: NodesRepository): GetCurrentNodeCase = repository

    @Provides
    fun provideGetNodeUrlCase(repository: NodesRepository): GetNodeUrlCase = repository

    @Provides
    fun provideGetNodesCase(repository: NodesRepository): GetNodesCase = repository

    @Provides
    fun provideAddNodeCase(repository: NodesRepository): AddNodeCase = repository

    @Provides
    fun provideDeleteNodeCase(repository: NodesRepository): DeleteNodeCase = repository
}
