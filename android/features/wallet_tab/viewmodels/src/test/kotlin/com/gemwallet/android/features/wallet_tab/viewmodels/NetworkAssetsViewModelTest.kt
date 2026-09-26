package com.gemwallet.android.features.wallet_tab.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onSubscription
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletHomeServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class NetworkAssetsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val native = mockAssetData(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18))
    private val pinnedToken =
        mockAssetData(
            asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7"), name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20),
            metadata = mockAssetMetaData(isPinned = true),
        )
    private val unpinnedToken = mockAssetData(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xusdc"), symbol = "USDC", type = AssetType.ERC20))
    private val hiddenToken = mockAssetData(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xdai"), symbol = "DAI", type = AssetType.ERC20))
    private val otherWalletToken = mockAssetData(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xlink"), symbol = "LINK", type = AssetType.ERC20))

    private val active = MutableStateFlow(listOf(native, pinnedToken, unpinnedToken))
    private val hidden = MutableStateFlow(listOf(hiddenToken))
    private val otherWalletActive = MutableStateFlow(listOf(native, otherWalletToken))
    private val otherWalletHidden = MutableStateFlow(emptyList<AssetData>())
    private var activeSubscriptions = 0

    private val walletId = MutableStateFlow(WalletId(FIRST_WALLET))

    private val assetsQuery = mockk<AssetsQuery> {
        every { this@mockk(any<WalletId>(), Chain.Ethereum) } answers {
            val assets = if (firstArg<WalletId>().id == FIRST_WALLET) active else otherWalletActive
            assets.onSubscription { activeSubscriptions += 1 }
        }
        every { hidden(any(), Chain.Ethereum) } answers {
            if (firstArg<WalletId>().id == FIRST_WALLET) hidden else otherWalletHidden
        }
    }

    private val service = mockk<GemWalletHomeServiceInterface>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `one snapshot feeds the pinned, unpinned and hidden groups`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        assertEquals(listOf(pinnedToken.asset.id), viewModel.pinned.first { it.isNotEmpty() }.map { it.id })
        assertEquals(listOf(unpinnedToken.asset.id), viewModel.unpinned.first { it.isNotEmpty() }.map { it.id })
        assertEquals(listOf(hiddenToken.asset.id), viewModel.hidden.first { it.isNotEmpty() }.map { it.id })
        assertEquals(1, activeSubscriptions)
    }

    @Test
    fun `the initial balance refresh asks for the active and hidden assets of the chain`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        viewModel.pinned.first { it.isNotEmpty() }
        advanceUntilIdle()

        coVerify(exactly = 1) {
            service.updateBalances(
                listOf(pinnedToken.asset.id.toIdentifier(), unpinnedToken.asset.id.toIdentifier(), hiddenToken.asset.id.toIdentifier()),
            )
        }
    }

    @Test
    fun `pinning reads the pinned group of the current snapshot`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        viewModel.pinned.first { it.isNotEmpty() }

        viewModel.togglePin(pinnedToken.asset.id)
        viewModel.togglePin(unpinnedToken.asset.id)
        advanceUntilIdle()

        coVerify(exactly = 1) { service.setAssetPinned(match { it.id == pinnedToken.asset.id.toIdentifier() }, false) }
        coVerify(exactly = 1) { service.setAssetPinned(match { it.id == unpinnedToken.asset.id.toIdentifier() }, true) }
    }

    @Test
    fun `a wallet change replaces the groups`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        viewModel.pinned.first { it.isNotEmpty() }

        walletId.value = WalletId(SECOND_WALLET)

        assertEquals(listOf(otherWalletToken.asset.id), viewModel.unpinned.first { it.map { item -> item.id } == listOf(otherWalletToken.asset.id) }.map { it.id })
        assertEquals(emptyList<Any>(), viewModel.pinned.first { it.isEmpty() })
        assertEquals(emptyList<Any>(), viewModel.hidden.first { it.isEmpty() })
        assertEquals(2, activeSubscriptions)
    }

    @Test
    fun `adding an asset whose first balance fetch fails shows the error`() = runTest(testDispatcher) {
        coEvery { service.setAssetsEnabled(any(), true) } throws RuntimeException("offline")
        val viewModel = createViewModel()
        val toast = CompletableDeferred<ToastMessage>()
        val collector = launch { toast.complete(viewModel.toastEvents.first()) }

        viewModel.addToWallet(hiddenToken.asset.id)

        assertEquals(R.drawable.ic_error, toast.await().image)
        collector.cancel()
    }

    private fun createViewModel() = NetworkAssetsViewModel(
        assetsQuery = assetsQuery,
        getCurrentWalletId = object : GetCurrentWalletId {
            override fun invoke(): Flow<WalletId> = walletId
        },
        getCurrentCurrency = object : GetCurrentCurrency {
            override fun getCurrency(): StateFlow<Currency> = MutableStateFlow(Currency.USD)
        },
        service = service,
        ioDispatcher = testDispatcher,
        context = mockk<Context> { every { getString(any()) } returns "Assets" },
        savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Chain.key to Chain.Ethereum.string)),
    )

    private companion object {
        const val FIRST_WALLET = "wallet-1"
        const val SECOND_WALLET = "wallet-2"
    }
}
