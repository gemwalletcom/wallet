package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetBlockExplorers
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetBlockExplorerCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.data.repositories.nodes.NodesRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.NodesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.Config

@InstallIn(SingletonComponent::class)
@Module
object NodesModule {

    @Provides
    @Singleton
    fun provideGemstoneConfig(): Config = Config()

    @Provides
    @Singleton
    fun provideNodesRepository(
        @ApplicationContext context: Context,
        nodesDao: NodesDao,
        config: Config,
    ): NodesRepository = NodesRepository(
        nodesDao = nodesDao,
        configStore = ConfigStore(
            context.getSharedPreferences(
                "node-config",
                Context.MODE_PRIVATE
            )
        ),
        config = config,
    )

    @Provides
    fun provideSetCurrentNodeCase(repository: NodesRepository): SetCurrentNodeCase = repository

    @Provides
    fun provideGetCurrentNodeCase(repository: NodesRepository): GetCurrentNodeCase = repository

    @Provides
    fun provideSetBlockExplorerCase(repository: NodesRepository): SetBlockExplorerCase = repository

    @Provides
    fun provideGetBlockExplorersCase(repository: NodesRepository): GetBlockExplorers = repository

    @Provides
    fun provideGetCurrentBlockExplorerCase(repository: NodesRepository): GetCurrentBlockExplorer = repository

    @Provides
    fun provideGetNodesCase(repository: NodesRepository): GetNodesCase = repository

    @Provides
    fun provideAddNodeCase(repository: NodesRepository): AddNodeCase = repository

    @Provides
    fun provideDeleteNodeCase(repository: NodesRepository): DeleteNodeCase = repository
}
