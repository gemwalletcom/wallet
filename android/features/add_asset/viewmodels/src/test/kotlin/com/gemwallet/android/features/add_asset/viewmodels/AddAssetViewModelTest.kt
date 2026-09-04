package com.gemwallet.android.features.add_asset.viewmodels

import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.add_asset.viewmodels.models.TokenSearchState
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAddAssetServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class AddAssetViewModelTest {

    private val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.Ethereum)))
    private val token = mockAsset(chain = Chain.Ethereum, tokenId = "0x1", name = "Token", symbol = "TKN", decimals = 18, type = AssetType.ERC20)
    private val service = mockk<GemAddAssetServiceInterface> {
        every { chains(any()) } returns listOf(Chain.Ethereum.string)
        every { defaultChain(any()) } returns Chain.Ethereum.string
        every { matchingChains(any(), any()) } answers { firstArg() }
        every { tokenUrl(any(), any()) } returns null
        coEvery { token(Chain.Ethereum.string, "0x1") } returns token.toGem()
        coEvery { add(any(), any()) } returns Unit
    }
    private val getSession = mockk<GetSession> { every { this@mockk() } returns MutableStateFlow(mockSession(wallet = wallet)) }

    @Before
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `typed address resolves the token through the service`() = runTest {
        val viewModel = AddAssetViewModel(getSession, service)
        try {
            viewModel.addressState.value = "0x1"
            Snapshot.sendApplyNotifications()

            assertEquals(TokenSearchState.Found(token), viewModel.searchState.first { it is TokenSearchState.Found })
            assertEquals(token, viewModel.token.first { it != null })
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `addAsset adds the found token to the current wallet`() = runTest {
        val viewModel = AddAssetViewModel(getSession, service)
        try {
            viewModel.addressState.value = "0x1"
            Snapshot.sendApplyNotifications()
            viewModel.token.first { it != null }

            var finished = false
            viewModel.addAsset { finished = true }.join()

            coVerify(exactly = 1) { service.add(wallet.toJson(), token.id.toIdentifier()) }
            assertEquals(true, finished)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }
}
