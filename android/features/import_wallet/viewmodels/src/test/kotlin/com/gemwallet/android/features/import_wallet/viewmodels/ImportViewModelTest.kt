package com.gemwallet.android.features.import_wallet.viewmodels

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.NameServiceMock
import com.gemwallet.android.testkit.mockNameRecord
import uniffi.gemstone.GemMnemonic
import uniffi.gemstone.GemNameServiceInterface
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.model.ImportType
import uniffi.gemstone.GemNameRecordState
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemWalletImportKind
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ImportViewModelTest {

    private val chain = Chain.Ethereum

    private fun viewModel(nameService: GemNameServiceInterface) = ImportViewModel(
        service = mockk(relaxed = true),
        nameService = nameService,
        mnemonic = GemMnemonic(),
        context = mockk<Context> {
            every { getString(any()) } returns "Wallet"
            every { getString(any(), *anyVararg()) } returns "Wallet"
        },

    )

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { any<Chain>().networkName() } returns "Ethereum"
    }

    @After
    fun tearDown() {
        unmockkAll()
        Dispatchers.resetMain()
    }

    @Test
    fun privateKeyInputNeverReachesTheResolver() = runTest {
        Dispatchers.setMain(StandardTestDispatcher(testScheduler))
        val addressInput = NameServiceMock()
        val viewModel = viewModel(addressInput)

        viewModel.importSelect(ImportType(GemWalletImportKind.PRIVATE_KEY, chain)).join()
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth")
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), addressInput.requests)
        assertEquals(GemNameRecordState.None, viewModel.nameResolveState.value)
    }

    @Test
    fun viewAddressInputResolves() = runTest {
        Dispatchers.setMain(StandardTestDispatcher(testScheduler))
        val addressInput = NameServiceMock()
        val viewModel = viewModel(addressInput)

        viewModel.importSelect(ImportType(GemWalletImportKind.ADDRESS, chain)).join()
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth")
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), addressInput.requests)
        assertEquals(GemNameRecordState.Complete(mockNameRecord().toGem()), viewModel.nameResolveState.value)
    }
}
