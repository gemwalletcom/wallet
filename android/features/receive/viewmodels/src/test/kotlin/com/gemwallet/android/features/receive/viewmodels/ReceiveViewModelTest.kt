package com.gemwallet.android.features.receive.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.receive.cases.GetReceiveAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemReceiveServiceInterface
import uniffi.gemstone.GemReceiveNetworks
import uniffi.gemstone.GemReceiveWarning

@OptIn(ExperimentalCoroutinesApi::class)
class ReceiveViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<androidx.lifecycle.ViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val bitcoin = mockAsset()
    private val ethereum = mockAssetEthereum()
    private val wallet = mockWallet(
        id = "multicoin_0xabc",
        accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1q"), mockAccount(chain = Chain.Ethereum, address = "0xabc")),
    )

    private fun receiveModel(
        service: GemReceiveServiceInterface,
        assets: Map<com.wallet.core.primitives.AssetId, AssetInfo> = mapOf(bitcoin.id to mockAssetInfo(asset = bitcoin)),
    ): ReceiveViewModel {
        val info: GetReceiveAssetInfo = mockk {
            every { this@mockk.invoke(any()) } answers { flowOf(assets[firstArg()]) }
        }
        val walletAssets: GetWalletAssets = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(assets.values.toList())
        }
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = wallet)) }
        return ReceiveViewModel(bitcoin.id, info, walletAssets, service, session, dispatcher, mockk(relaxed = true)).also { models.add(it) }
    }

    @Test
    fun `the networks and the warnings both come from Core`() = runTest(dispatcher) {
        val service: GemReceiveServiceInterface = mockk(relaxed = true) {
            every { networks(any(), any(), any()) } returns GemReceiveNetworks(listOf(bitcoin.id.toIdentifier(), ethereum.id.toIdentifier()), showsSelector = true)
            every { warnings(Chain.Bitcoin.string) } returns listOf(GemReceiveWarning.NO_MEMO_REQUIRED)
            every { warnings(Chain.Ethereum.string) } returns emptyList()
        }
        val model = receiveModel(service)

        assertEquals(listOf(bitcoin.id.toIdentifier(), ethereum.id.toIdentifier()), model.networks.first { it.showsSelector }.assetIds)
        coVerify { service.syncNetworks(bitcoin.id.toIdentifier(), wallet.toGem()) }
        assertEquals(listOf(GemReceiveWarning.NO_MEMO_REQUIRED), model.warnings(Chain.Bitcoin))
        assertEquals(emptyList<GemReceiveWarning>(), model.warnings(Chain.Ethereum))
    }

    @Test
    fun `picking another network swaps the asset the screen shows`() = runTest(dispatcher) {
        val service: GemReceiveServiceInterface = mockk(relaxed = true) {
            every { networks(any(), any(), any()) } returns GemReceiveNetworks(listOf(bitcoin.id.toIdentifier()), showsSelector = false)
        }
        val model = receiveModel(
            service,
            assets = mapOf(bitcoin.id to mockAssetInfo(asset = bitcoin), ethereum.id to mockAssetInfo(asset = ethereum)),
        )
        model.asset.first { it?.asset?.id == bitcoin.id }

        model.selectAsset(ethereum.id)

        assertEquals(ethereum.id, model.asset.first { it?.asset?.id == ethereum.id }?.asset?.id)
    }

    @Test
    fun `showing the screen enables the asset for the wallet`() = runTest(dispatcher) {
        val service: GemReceiveServiceInterface = mockk(relaxed = true) {
            every { networks(any(), any(), any()) } returns GemReceiveNetworks(listOf(bitcoin.id.toIdentifier()), showsSelector = false)
        }
        val model = receiveModel(service)
        model.asset.first { it != null }

        model.setVisible().join()

        coVerify { service.enableAsset(wallet.id.id, bitcoin.id.toIdentifier()) }
    }
}
