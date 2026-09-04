package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemConfirmInitialState
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferService
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelHeaderTest {

    private val transferService = GemTransferService()
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
        val viewModel = viewModel(transfer(value))

        val amount = viewModel.amountUIModel.first { it != null }

        assertEquals(asset, amount?.asset)
        assertEquals(value, amount?.amount)
        assertEquals(FeeUIModel.Calculating, viewModel.feeUIModel.first { it != null })
        assertTrue(viewModel.state.value is ConfirmState.Prepare)

        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
    }

    @Test
    fun maxSendHeaderCarriesTheRequestedBalanceWhileTheFeeIsStillLoading() = runTest(testDispatcher) {
        val balance = BigInteger.valueOf(170_400)
        val viewModel = viewModel(transfer(balance, useMaxAmount = true))

        val amount = viewModel.amountUIModel.first { it != null }

        assertEquals(asset, amount?.asset)
        assertEquals(balance, amount?.amount)

        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
    }

    private fun transfer(value: BigInteger, useMaxAmount: Boolean = false) = GemTransferData(
        inputType = GemTransactionInputType.Transfer(asset.toGem()),
        recipient = GemRecipient(address = "bc1qrecipient"),
        value = value,
        useMaxAmount = useMaxAmount,
    )

    private fun viewModel(transfer: GemTransferData): ConfirmViewModel {
        every { confirmService.getCurrency() } returns Currency.USD.toGem()
        every { confirmService.confirmInput(any(), transfer) } returns GemConfirmInput(from = account.toGem(), transfer = transfer)
        every { confirmService.initialState(any(), any()) } returns mockGemConfirmInitialState(asset)
        coEvery { confirmService.load(any(), any(), any()) } coAnswers { awaitCancellation() }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            buildConfirmProperties = mockk(relaxed = true),
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transferService.pack(transfer)))),
            transferService = transferService,
        )
    }
}
