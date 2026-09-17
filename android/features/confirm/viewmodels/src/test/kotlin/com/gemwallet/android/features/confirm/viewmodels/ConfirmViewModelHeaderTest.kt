package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.job
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemTransferData
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelHeaderTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset()
    private val account = mockAccount(chain = Chain.Bitcoin)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun headerCarriesTheRequestedAmountWhileTheFeeIsStillLoading() = runTest(testDispatcher) {
        val value = BigInteger.valueOf(150_000)
        val viewModel = viewModel(mockGemTransferData(value = value))

        val amount = viewModel.amountUIModel.first { it != null }

        assertEquals(asset, amount?.asset)
        assertEquals(value, amount?.amount)
        assertEquals(FeeUIModel.Calculating, viewModel.feeUIModel.first { it != null })
        assertEquals(GemConfirmPhase.LOADING, viewModel.screen.value.phase)

        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
    }

    @Test
    fun maxSendHeaderCarriesTheRequestedBalanceWhileTheFeeIsStillLoading() = runTest(testDispatcher) {
        val balance = BigInteger.valueOf(170_400)
        val viewModel = viewModel(mockGemTransferData(value = balance, useMaxAmount = true))

        val amount = viewModel.amountUIModel.first { it != null }

        assertEquals(asset, amount?.asset)
        assertEquals(balance, amount?.amount)

        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
    }

    private fun viewModel(transfer: GemTransferData): ConfirmViewModel {
        val confirmation = mockk<GemConfirmation>()
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.insufficientNetworkFeeBuyAmount() } returns 10
        every { confirmation.screen() } returns mockGemConfirmScreen()
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset)
        coEvery { confirmation.load(any()) } coAnswers { awaitCancellation() }
        every { confirmService.confirmation(any(), transfer, any()) } returns confirmation
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            buildConfirmProperties = mockk(relaxed = true),
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))),
            context = mockk<Context> { every { getString(any()) } returns "Error"; every { getString(any(), *anyVararg()) } returns "Error" },
        )
    }
}
