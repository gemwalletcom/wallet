package com.gemwallet.android.features.transfer.viewmodels.confirm

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.ToastPresenter
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Account
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.job
import kotlinx.coroutines.test.TestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.runner.Description
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemTransferData

class ConfirmTransferRule(dispatcher: TestDispatcher, private val account: Account) : MainDispatcherRule(dispatcher) {

    val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    val toastPresenter = ToastPresenter()
    private var model: ConfirmTransferViewModel? = null

    fun viewModel(transfer: GemTransferData) = viewModel(SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))))

    fun viewModel(handle: SavedStateHandle) = ConfirmTransferViewModel(
        getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(account))))
        },
        confirmService = confirmService,
        savedStateHandle = handle,
        observeRefreshInterval = mockk(relaxed = true),
        toastPresenter = toastPresenter,
        ioDispatcher = dispatcher,
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },
    ).also { model = it }

    override fun finished(description: Description) {
        runTest(dispatcher) { model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin() }
        super.finished(description)
    }
}
