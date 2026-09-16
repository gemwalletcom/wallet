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
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Account
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
import uniffi.gemstone.GemNftServiceInterface
import uniffi.gemstone.GemReceiveServiceInterface
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
    private val ethereum = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
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
        return ReceiveViewModel(bitcoin.id, info, walletAssets, service, session).also { models.add(it) }
    }

    @Test
    fun `the networks and the warnings both come from Core`() = runTest(dispatcher) {
        val service: GemReceiveServiceInterface = mockk(relaxed = true) {
            every { networkAssetIds(any(), any(), any()) } returns listOf(bitcoin.id.toIdentifier(), ethereum.id.toIdentifier())
            every { warnings(Chain.Bitcoin.string) } returns listOf(GemReceiveWarning.NO_MEMO_REQUIRED)
            every { warnings(Chain.Ethereum.string) } returns emptyList()
        }
        val model = receiveModel(service)

        assertEquals(listOf(bitcoin.id, ethereum.id), model.networkAssetIds.first { it.size == 2 })
        coVerify { service.syncNetworkAssetIds(bitcoin.id.toIdentifier(), wallet.toGem()) }
        assertEquals(listOf(GemReceiveWarning.NO_MEMO_REQUIRED), model.warnings(Chain.Bitcoin))
        assertEquals(emptyList<GemReceiveWarning>(), model.warnings(Chain.Ethereum))
    }

    @Test
    fun `picking another network swaps the asset the screen shows`() = runTest(dispatcher) {
        val service: GemReceiveServiceInterface = mockk(relaxed = true) {
            every { networkAssetIds(any(), any(), any()) } returns listOf(bitcoin.id.toIdentifier())
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
            every { networkAssetIds(any(), any(), any()) } returns listOf(bitcoin.id.toIdentifier())
        }
        val model = receiveModel(service)
        model.asset.first { it != null }

        model.setVisible().join()

        coVerify { service.enableAsset(wallet.id.id, bitcoin.id.toIdentifier()) }
    }

    @Test
    fun `the nft receive chains and their addresses come from Core`() = runTest(dispatcher) {
        val accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc"))
        val service: GemNftServiceInterface = mockk(relaxed = true) {
            every { receiveAccounts(any(), any()) } returns accounts.map(Account::toGem)
        }
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = wallet)) }
        val model = ReceiveNftChainsViewModel(session, service).also { models.add(it) }

        assertEquals(listOf(Chain.Ethereum), model.chains.first { it.isNotEmpty() })
        assertEquals("0xabc", model.addressFor(Chain.Ethereum))
        assertEquals("", model.addressFor(Chain.Bitcoin))
    }
}
