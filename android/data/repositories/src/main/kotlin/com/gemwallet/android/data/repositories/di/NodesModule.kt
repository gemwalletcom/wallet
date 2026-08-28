package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.data.repositories.gemstone.GemstonePreferencesStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.flow.flowOf
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
    fun provideSetCurrentNodeCase(
        nodeService: GemNodeService,
    ): SetCurrentNodeCase = object : SetCurrentNodeCase {
        override suspend fun setCurrentNode(chain: Chain, node: Node) =
            nodeService.selectNode(chain.string, node.url)
    }

    @Provides
    fun provideGetCurrentNodeCase(
        nodeService: GemNodeService,
    ): GetCurrentNodeCase = object : GetCurrentNodeCase {
        override fun getCurrentNode(chain: Chain) =
            nodeService.selectedNode(chain.string).decodeJson<Node>()
    }

    @Provides
    fun provideGetNodeUrlCase(
        nodeService: GemNodeService,
    ): GetNodeUrlCase = object : GetNodeUrlCase {
        override fun getNodeUrl(chain: Chain) = nodeService.nodeUrl(chain.string)
    }

    @Provides
    fun provideGetNodesCase(
        nodeService: GemNodeService,
    ): GetNodesCase = object : GetNodesCase {
        override suspend fun getNodes(chain: Chain) = flowOf(
            nodeService.sortedNodes(chain.string, nodeService.getNodes(chain.string)).map { it.decodeJson<Node>() },
        )

        override fun canDeleteNode(chain: Chain, url: String) =
            nodeService.canDeleteNode(chain.string, url)

        override fun getDefaultNodes(chain: Chain) =
            nodeService.getDefaultNodes(chain.string).map { it.decodeJson<Node>() }
    }

    @Provides
    fun provideAddNodeCase(
        nodeService: GemNodeService,
    ): AddNodeCase = object : AddNodeCase {
        override suspend fun addNode(chain: Chain, url: String) =
            nodeService.addNode(chain.string, url)
    }

    @Provides
    fun provideDeleteNodeCase(
        nodeService: GemNodeService,
    ): DeleteNodeCase = object : DeleteNodeCase {
        override suspend fun deleteNode(chain: Chain, node: Node) =
            nodeService.deleteNode(chain.string, node.url)
    }
}
