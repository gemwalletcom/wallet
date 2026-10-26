package com.gemwallet.android.features.onboarding.viewmodels.import_wallet

import android.content.Context
import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.ui.text.TextRange
import com.gemwallet.android.application.device.cases.EnablePushForNewWallet
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.testkit.MainDispatcherRule
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
import io.mockk.verify
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletImportException
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportRequest
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletImportScreen
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class ImportWalletViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

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

    private fun viewModel(
        importType: ImportType,
        nameService: GemNameServiceInterface,
        ioDispatcher: CoroutineDispatcher,
        service: GemWalletServiceInterface = service(),
        enablePushForNewWallet: EnablePushForNewWallet = mockk(relaxed = true),
    ) = ImportWalletViewModel(
        importType = importType,
        service = service,
        nameService = nameService,
        enablePushForNewWallet = enablePushForNewWallet,
        ioDispatcher = ioDispatcher,
        context = mockk<Context> {
            every { getString(any()) } returns "Wallet"
            every { getString(any(), *anyVararg()) } returns "Wallet"
        },
    )

    private fun ImportWalletViewModel.type(value: String, cursor: Int = value.length) {
        inputState.edit {
            replace(0, length, value)
            selection = TextRange(cursor)
        }
        Snapshot.sendApplyNotifications()
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun privateKeyInputNeverReachesTheResolver() = runTest(dispatcher) {
        val addressInput = NameServiceMock()
        val viewModel = viewModel(ImportType(GemWalletImportKind.PRIVATE_KEY, chain), addressInput, dispatcher)

        viewModel.type("vitalik.eth")
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), addressInput.requests)
        assertEquals(GemNameRecordState.None, viewModel.nameResolveState.value)
    }

    @Test
    fun viewAddressInputResolves() = runTest(dispatcher) {
        val addressInput = NameServiceMock()
        val viewModel = viewModel(ImportType(GemWalletImportKind.ADDRESS, chain), addressInput, dispatcher)

        viewModel.type("vitalik.eth")
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), addressInput.requests)
        assertEquals(GemNameRecordState.Complete(mockNameRecord(name = "vitalik.eth", chain = chain.toGem())), viewModel.nameResolveState.value)
    }

    @Test
    fun anImportTappedAsTheScreenOpensIsStillNamed() = runTest(dispatcher) {
        val service = service()
        val viewModel = viewModel(ImportType(GemWalletImportKind.PHRASE, chain), NameServiceMock(), dispatcher, service)
        val request = slot<GemWalletImportRequest>()

        viewModel.type("abandon ability")
        viewModel.import { }
        advanceUntilIdle()

        coVerify { service.importWallet(capture(request)) }
        assertEquals("Wallet", request.captured.defaultName)
        assertEquals("abandon ability", request.captured.input)
    }

    @Test
    fun continuingWithAnExistingWalletOffersPush() = runTest(dispatcher) {
        val service = service()
        coEvery { service.importWallet(any()) } returns GemWalletImportResult.Existing(mockWallet(name = "Main").toGem())
        val enablePush = mockk<EnablePushForNewWallet>(relaxed = true)
        val onImported = mockk<() -> Unit>(relaxed = true)
        val viewModel = viewModel(ImportType(GemWalletImportKind.PHRASE, chain), NameServiceMock(), dispatcher, service, enablePush)

        viewModel.type("abandon ability")
        viewModel.import(onImported)
        advanceUntilIdle()

        assertEquals("Main", viewModel.uiState.value.existingWalletName)
        verify(exactly = 0) { enablePush.enablePushForNewWallet() }

        viewModel.continueExistingWallet(onImported)
        advanceUntilIdle()

        assertNull(viewModel.uiState.value.existingWalletName)
        verify(exactly = 1) { enablePush.enablePushForNewWallet() }
        verify(exactly = 1) { onImported() }
    }

    @Test
    fun aSuggestionCompletesTheLastWord() = runTest(dispatcher) {
        val viewModel = viewModel(ImportType(GemWalletImportKind.PHRASE, chain), NameServiceMock(), dispatcher)

        viewModel.type("abandon woo")
        advanceUntilIdle()

        assertEquals(listOf("wood", "wool"), viewModel.suggestions.value)
        viewModel.selectSuggestion("wood")
        Snapshot.sendApplyNotifications()
        advanceUntilIdle()
        assertEquals("abandon wood ", viewModel.inputState.text.toString())
        assertEquals(emptyList<String>(), viewModel.suggestions.value)
    }

    @Test
    fun aCursorInsideThePhraseHidesSuggestions() = runTest(dispatcher) {
        val viewModel = viewModel(ImportType(GemWalletImportKind.PHRASE, chain), NameServiceMock(), dispatcher)

        viewModel.type("abandon woo", 3)
        advanceUntilIdle()

        assertEquals(emptyList<String>(), viewModel.suggestions.value)
    }

    @Test
    fun aRepeatedFailureShowsItsErrorAgain() = runTest(dispatcher) {
        val service = service()
        coEvery { service.importWallet(any()) } throws GemServiceException.WalletImport(GemWalletImportException.InvalidAddress())
        val viewModel = viewModel(ImportType(GemWalletImportKind.ADDRESS, chain), NameServiceMock(), dispatcher, service)

        viewModel.type("0x123")
        viewModel.import { }
        advanceUntilIdle()
        val error = viewModel.uiState.value.dataError
        assertNotNull(error)

        viewModel.type("0x1234")
        advanceUntilIdle()
        assertNull(viewModel.uiState.value.dataError)

        viewModel.import { }
        advanceUntilIdle()
        assertEquals(error, viewModel.uiState.value.dataError)
    }

    @Test
    fun onlyASingleChainImportOffersScanning() {
        assertFalse(ImportWalletUIState(importType = ImportType.phrase()).showsScan)
        assertTrue(ImportWalletUIState(importType = ImportType.phrase(chain)).showsScan)
    }
}
