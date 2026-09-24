package com.gemwallet.android.features.import_wallet.viewmodels

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.testkit.NameServiceMock
import com.gemwallet.android.testkit.mockNameRecord
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import io.mockk.unmockkAll
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportRequest
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletImportScreen
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class ImportViewModelTest {

    private val chain = Chain.Ethereum

    private fun service() = mockk<GemWalletServiceInterface>(relaxed = true) {
        coEvery { defaultWalletName(any()) } returns GemLocalizedText.WalletDefaultName(index = 1)
        coEvery { importWallet(any()) } returns GemWalletImportResult.New(mockWallet().toGem())
        every { importScreen(any()) } returns GemWalletImportScreen(
            title = GemLocalizedText.WalletMulticoin,
            kinds = listOf(GemWalletImportKind.PHRASE, GemWalletImportKind.PRIVATE_KEY, GemWalletImportKind.ADDRESS),
            showsKinds = true,
        )
    }

    private fun viewModel(nameService: GemNameServiceInterface, ioDispatcher: CoroutineDispatcher, service: GemWalletServiceInterface = service()) = ImportViewModel(
        service = service,
        nameService = nameService,
        ioDispatcher = ioDispatcher,
        context = mockk<Context> {
            every { getString(any()) } returns "Wallet"
            every { getString(any(), *anyVararg()) } returns "Wallet"
        },
    )

    @After
    fun tearDown() {
        unmockkAll()
        Dispatchers.resetMain()
    }

    @Test
    fun privateKeyInputNeverReachesTheResolver() = runTest {
        val dispatcher = StandardTestDispatcher(testScheduler)
        Dispatchers.setMain(dispatcher)
        val addressInput = NameServiceMock()
        val viewModel = viewModel(addressInput, dispatcher)

        viewModel.importSelect(ImportType(GemWalletImportKind.PRIVATE_KEY, chain))
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth", 11)
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), addressInput.requests)
        assertEquals(GemNameRecordState.None, viewModel.nameResolveState.value)
    }

    @Test
    fun viewAddressInputResolves() = runTest {
        val dispatcher = StandardTestDispatcher(testScheduler)
        Dispatchers.setMain(dispatcher)
        val addressInput = NameServiceMock()
        val viewModel = viewModel(addressInput, dispatcher)

        viewModel.importSelect(ImportType(GemWalletImportKind.ADDRESS, chain))
        advanceUntilIdle()
        viewModel.onInput("vitalik.eth", 11)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), addressInput.requests)
        assertEquals(GemNameRecordState.Complete(mockNameRecord().toGem()), viewModel.nameResolveState.value)
    }

    @Test
    fun anImportTappedAsTheScreenOpensIsStillNamed() = runTest {
        val dispatcher = StandardTestDispatcher(testScheduler)
        Dispatchers.setMain(dispatcher)
        val service = service()
        val viewModel = viewModel(NameServiceMock(), dispatcher, service)
        val request = slot<GemWalletImportRequest>()

        viewModel.importSelect(ImportType(GemWalletImportKind.PHRASE, chain))
        viewModel.onInput("abandon ability", 15)
        viewModel.import { }
        advanceUntilIdle()

        coVerify { service.importWallet(capture(request)) }
        assertEquals("Wallet", request.captured.defaultName)
        assertEquals("abandon ability", request.captured.input)
    }
}
