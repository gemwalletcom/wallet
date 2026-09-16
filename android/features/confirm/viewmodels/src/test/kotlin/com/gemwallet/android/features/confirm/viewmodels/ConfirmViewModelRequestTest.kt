package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmButtonState
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelRequestTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
    private val account = mockAccount(chain = Chain.Ethereum)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation>(relaxed = true)
    private var model: ConfirmViewModel? = null

    private fun transfer(memo: String?) = GemTransferData(
        inputType = TransactionInputType.Transfer(asset.toGem()),
        recipient = GemRecipient(address = account.address, memo = memo),
        value = BigInteger.TEN,
    )

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun malformedParamsNeverBecomeARequest() = runTest(testDispatcher) {
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to "not a packed payload"))
        val viewModel = viewModel(handle).also { model = it }

        assertNull(viewModel.title.first())
        assertEquals(GemConfirmPhase.LOADING, viewModel.screen.value.phase)
        assertEquals(GemConfirmButtonState.LOADING, viewModel.button.value.state)
        verify(exactly = 0) { confirmService.confirmation(any(), any(), any()) }
    }

    @Test
    fun aLargePayloadIsDecodedIntoTheRequest() = runTest(testDispatcher) {
        val transfer = transfer("m".repeat(64 * 1024))
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack())))
        model = viewModel(handle)

        verify(timeout = AWAIT_MS, exactly = 1) { confirmService.confirmation(any(), transfer, any()) }
    }

    @Test
    fun theLatestParamsReplaceTheEarlierOnes() = runTest(testDispatcher) {
        val first = transfer("first")
        val second = transfer("second")
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(first.pack())))
        model = viewModel(handle)
        verify(timeout = AWAIT_MS) { confirmService.confirmation(any(), first, any()) }

        handle[RouteArgument.Params.key] = requireNotNull(second.pack())

        verify(timeout = AWAIT_MS) { confirmService.confirmation(any(), second, any()) }
    }

    private companion object {
        const val AWAIT_MS = 5_000L
    }

    private fun viewModel(handle: SavedStateHandle): ConfirmViewModel {
        every { confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns GemConfirmScreen(GemConfirmPhase.LOADING, false, null)
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset, preload = null)
        coEvery { confirmation.load(any()) } throws IllegalStateException("preload failed")
        return confirmViewModel(handle)
    }

    private fun confirmViewModel(handle: SavedStateHandle) = ConfirmViewModel(
        getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(account))))
        },
        buildConfirmProperties = mockk(relaxed = true),
        confirmService = confirmService,
        savedStateHandle = handle,
    )
}
